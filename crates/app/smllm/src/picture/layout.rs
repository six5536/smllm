//! A layered layout of a flat machine: states in rows from the initial state
//! down, each edge label in a row between two rows of states, and each long
//! or backward edge routed through placeholders so that it passes between
//! the boxes (CLI-8).
//!
//! The layout follows the usual four steps of a layered drawing: rank the
//! states by their distance from the initial state, split every edge into
//! steps of one row, order each row to cut crossings, and place each row
//! from the positions of its neighbours.

use crate::picture::{font, route};

/// The font size of a state's name.
pub(crate) const NAME_SIZE: f64 = 14.0;
/// The font size of a state's tags.
pub(crate) const TAG_SIZE: f64 = 11.0;
/// The font size of an edge's label.
pub(crate) const LABEL_SIZE: f64 = 12.0;
/// The height of one line of an edge's label.
pub(crate) const LINE: f64 = 15.0;
/// The space between the line of an edge and its label.
pub(crate) const LABEL_GAP: f64 = 5.0;

/// The space around the drawing.
pub(crate) const MARGIN: f64 = 16.0;
/// The diameter of the start dot.
pub(crate) const START: f64 = 14.0;
/// How far a loop reaches to the right of its box.
pub(crate) const LOOP_OUT: f64 = 28.0;
const GAP: f64 = 22.0;
const PAD: f64 = 12.0;
const SWEEPS: usize = 8;

/// A point.
pub(crate) type Pt = (f64, f64);

/// A state to draw.
pub(crate) struct StateSpec {
    /// The name.
    pub name: String,
    /// The tags, joined for the second line.
    pub tags: String,
}

/// An edge to draw, from one state to another or, with `to` `None`, back to
/// its source.
pub(crate) struct EdgeSpec {
    /// The index of the source state.
    pub from: usize,
    /// The index of the target state, `None` for a loop.
    pub to: Option<usize>,
    /// The lines of the label: the event, then the guard, if any.
    pub lines: Vec<String>,
}

/// A machine to lay out: its states, its initial state, and its edges.
pub(crate) struct Spec {
    /// The states, in file order.
    pub states: Vec<StateSpec>,
    /// The index of the initial state.
    pub initial: usize,
    /// The edges.
    pub edges: Vec<EdgeSpec>,
}

/// A box: its left, its top, its width, and its height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The drawing of an edge: an SVG path that ends at its target, and the
/// top left of its label.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Route {
    pub d: String,
    pub label: Pt,
}

/// A laid-out machine.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Layout {
    pub width: f64,
    pub height: f64,
    /// The centre of the start dot.
    pub start: Pt,
    /// The path from the start dot to the initial state.
    pub start_route: String,
    /// The box of each state, in the order of [`Spec::states`].
    pub boxes: Vec<Rect>,
    /// The route of each edge, in the order of [`Spec::edges`].
    pub routes: Vec<Route>,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Start,
    State(usize),
    Label(usize),
    Dummy,
}

/// A node of a row: a state, the start dot, a label, or a placeholder.
pub(crate) struct Node {
    pub kind: Kind,
    pub row: usize,
    /// The distance from the anchor to the left and the right of the slot.
    pub left: f64,
    pub right: f64,
    pub h: f64,
    pub x: f64,
}

/// The width and the height of a label of `lines`.
pub(crate) fn label_size(lines: &[String]) -> (f64, f64) {
    let w = lines
        .iter()
        .map(|l| font::width(l, LABEL_SIZE))
        .fold(0.0, f64::max);
    (w + LABEL_GAP + 5.0, LINE * lines.len() as f64)
}

/// Each state's rank: its distance from the initial state, and 0 for a state
/// that the initial state doesn't reach, with its own distances below it.
fn ranks(spec: &Spec) -> Vec<usize> {
    let n = spec.states.len();
    let mut rank = vec![usize::MAX; n];
    let roots = std::iter::once(spec.initial).chain(0..n);
    for root in roots {
        if rank[root] != usize::MAX {
            continue;
        }
        rank[root] = 0;
        let mut queue = std::collections::VecDeque::from([root]);
        while let Some(u) = queue.pop_front() {
            for e in &spec.edges {
                if let Some(v) = e.to
                    && e.from == u
                    && rank[v] == usize::MAX
                {
                    rank[v] = rank[u] + 1;
                    queue.push_back(v);
                }
            }
        }
    }
    rank
}

/// The row of a state of `rank`: row 0 holds the start dot, and every odd
/// row holds labels.
fn row_of(rank: usize) -> usize {
    2 * rank + 2
}

/// Lay out `spec`.
pub(crate) fn layout(spec: &Spec) -> Layout {
    let rank = ranks(spec);
    let mut nodes: Vec<Node> = Vec::new();
    fn add(nodes: &mut Vec<Node>, kind: Kind, row: usize, left: f64, right: f64, h: f64) -> usize {
        nodes.push(Node {
            kind,
            row,
            left,
            right,
            h,
            x: 0.0,
        });
        nodes.len() - 1
    }
    let start = add(&mut nodes, Kind::Start, 0, START / 2.0, START / 2.0, START);
    // States in rank order, so that each row starts in the order of a walk.
    let mut by_rank: Vec<usize> = (0..spec.states.len()).collect();
    by_rank.sort_by_key(|&i| (rank[i], i != spec.initial));
    let mut state_node = vec![0; spec.states.len()];
    for &i in &by_rank {
        let s = &spec.states[i];
        let loops: Vec<&EdgeSpec> = spec
            .edges
            .iter()
            .filter(|e| e.from == i && e.to.is_none())
            .collect();
        let (w, h) = box_size(s, &loops);
        let loop_w = loops
            .iter()
            .map(|e| label_size(&e.lines).0)
            .fold(0.0, f64::max);
        let extra = if loops.is_empty() {
            0.0
        } else {
            LOOP_OUT + loop_w
        };
        state_node[i] = add(
            &mut nodes,
            Kind::State(i),
            row_of(rank[i]),
            w / 2.0,
            w / 2.0 + extra,
            h,
        );
    }
    // Each edge as a path of nodes, one row at a time.
    let mut paths: Vec<Vec<usize>> = Vec::new();
    let first = row_of(rank[spec.initial]) - 1;
    let d = add(&mut nodes, Kind::Dummy, first, 0.0, 0.0, 0.0);
    paths.push(vec![start, d, state_node[spec.initial]]);
    let mut edge_path = vec![None; spec.edges.len()];
    for (k, e) in spec.edges.iter().enumerate() {
        let Some(to) = e.to else { continue };
        let (ru, rv) = (row_of(rank[e.from]), row_of(rank[to]));
        let (lw, lh) = label_size(&e.lines);
        let mut path = vec![state_node[e.from]];
        let rows: Vec<usize> = if rv > ru {
            (ru + 1..rv).collect()
        } else if rv < ru {
            (rv + 1..ru).rev().collect()
        } else {
            vec![ru + 1]
        };
        for (j, row) in rows.into_iter().enumerate() {
            let n = if j == 0 {
                add(&mut nodes, Kind::Label(k), row, 0.0, lw, lh)
            } else {
                add(&mut nodes, Kind::Dummy, row, 0.0, 0.0, 0.0)
            };
            path.push(n);
        }
        path.push(state_node[to]);
        edge_path[k] = Some(paths.len());
        paths.push(path);
    }
    let n_rows = nodes.iter().map(|n| n.row).max().unwrap_or(0) + 1;
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); n_rows];
    for (i, n) in nodes.iter().enumerate() {
        rows[n.row].push(i);
    }
    // Segments between adjacent rows, as (upper, lower).
    let mut segs: Vec<(usize, usize)> = Vec::new();
    for p in &paths {
        for w in p.windows(2) {
            let (a, b) = (w[0], w[1]);
            segs.push(if nodes[a].row < nodes[b].row {
                (a, b)
            } else {
                (b, a)
            });
        }
    }
    order(&mut rows, &nodes, &segs);
    place(&rows, &mut nodes, &segs);
    route::geometry(spec, &nodes, n_rows, &state_node, &paths, &edge_path)
}

fn box_size(s: &StateSpec, loops: &[&EdgeSpec]) -> (f64, f64) {
    let text = font::width(&s.name, NAME_SIZE).max(font::width(&s.tags, TAG_SIZE));
    let w = (text + 2.0 * PAD).max(72.0);
    let h: f64 = if s.tags.is_empty() { 32.0 } else { 46.0 };
    let loops_h: f64 = loops.iter().map(|e| label_size(&e.lines).1 + 6.0).sum();
    (w, h.max(loops_h + 6.0))
}

/// The number of crossings between the segments of adjacent rows.
fn crossings(pos: &[usize], segs: &[(usize, usize)]) -> usize {
    let mut n = 0;
    for (i, &(a1, b1)) in segs.iter().enumerate() {
        for &(a2, b2) in &segs[i + 1..] {
            let da = pos[a1] as isize - pos[a2] as isize;
            let db = pos[b1] as isize - pos[b2] as isize;
            if da * db < 0 {
                n += 1;
            }
        }
    }
    n
}

/// Order each row by the mean position of its neighbours, sweeping down and
/// up, and keep the order with the fewest crossings.
fn order(rows: &mut [Vec<usize>], nodes: &[Node], segs: &[(usize, usize)]) {
    let n = nodes.len();
    let positions = |rows: &[Vec<usize>]| {
        let mut pos = vec![0; n];
        for r in rows {
            for (i, &v) in r.iter().enumerate() {
                pos[v] = i;
            }
        }
        pos
    };
    // Segments only join adjacent rows, so the row pair of a crossing is that
    // of its two segments: count each row pair apart.
    let n_rows = rows.len();
    let count = |pos: &[usize]| {
        let mut total = 0;
        for r in 0..n_rows.saturating_sub(1) {
            let here: Vec<(usize, usize)> = segs
                .iter()
                .copied()
                .filter(|&(a, _)| nodes[a].row == r)
                .collect();
            total += crossings(pos, &here);
        }
        total
    };
    let mut best = rows.to_vec();
    let mut best_n = count(&positions(rows));
    for sweep in 0..SWEEPS {
        let down = sweep % 2 == 0;
        let order: Vec<usize> = if down {
            (1..rows.len()).collect()
        } else {
            (0..rows.len().saturating_sub(1)).rev().collect()
        };
        for r in order {
            let pos = positions(rows);
            let key = |v: usize| {
                let ns: Vec<usize> = segs
                    .iter()
                    .filter_map(|&(a, b)| match (down, a == v, b == v) {
                        (true, _, true) => Some(a),
                        (false, true, _) => Some(b),
                        _ => None,
                    })
                    .collect();
                if ns.is_empty() {
                    pos[v] as f64
                } else {
                    ns.iter().map(|&u| pos[u] as f64).sum::<f64>() / ns.len() as f64
                }
            };
            let mut keyed: Vec<(f64, usize, usize)> =
                rows[r].iter().map(|&v| (key(v), pos[v], v)).collect();
            keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            rows[r] = keyed.into_iter().map(|(_, _, v)| v).collect();
        }
        let c = count(&positions(rows));
        if c < best_n {
            best_n = c;
            best = rows.to_vec();
        }
    }
    rows.clone_from_slice(&best);
}

/// Place the nodes of each row in their order, each as near the mean of its
/// neighbours as the gaps allow, by averaging a placement packed from the
/// left with one packed from the right.
fn place(rows: &[Vec<usize>], nodes: &mut [Node], segs: &[(usize, usize)]) {
    let sep = |nodes: &[Node], a: usize, b: usize| {
        let gap = if nodes[a].kind == Kind::Dummy && nodes[b].kind == Kind::Dummy {
            GAP / 2.0
        } else {
            GAP
        };
        nodes[a].right + gap + nodes[b].left
    };
    for r in rows {
        let mut x = 0.0;
        for (i, &v) in r.iter().enumerate() {
            if i > 0 {
                x += sep(nodes, r[i - 1], v);
            }
            nodes[v].x = x;
        }
    }
    for sweep in 0..2 * SWEEPS {
        let down = sweep % 2 == 0;
        let order: Vec<usize> = if down {
            (1..rows.len()).collect()
        } else {
            (0..rows.len().saturating_sub(1)).rev().collect()
        };
        for r in order {
            let row = &rows[r];
            if row.is_empty() {
                continue;
            }
            let want: Vec<f64> = row
                .iter()
                .map(|&v| {
                    let xs: Vec<f64> = segs
                        .iter()
                        .filter_map(|&(a, b)| match (down, a == v, b == v) {
                            (true, _, true) => Some(nodes[a].x),
                            (false, true, _) => Some(nodes[b].x),
                            _ => None,
                        })
                        .collect();
                    if xs.is_empty() {
                        nodes[v].x
                    } else {
                        xs.iter().sum::<f64>() / xs.len() as f64
                    }
                })
                .collect();
            let n = row.len();
            let mut left = want.clone();
            for i in 1..n {
                left[i] = left[i].max(left[i - 1] + sep(nodes, row[i - 1], row[i]));
            }
            let mut right = want;
            for i in (0..n - 1).rev() {
                right[i] = right[i].min(right[i + 1] - sep(nodes, row[i], row[i + 1]));
            }
            for i in 0..n {
                nodes[row[i]].x = (left[i] + right[i]) / 2.0;
            }
        }
    }
    let min = nodes
        .iter()
        .map(|n| n.x - n.left)
        .fold(f64::INFINITY, f64::min);
    for n in nodes.iter_mut() {
        n.x += MARGIN - min;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(states: &[&str], edges: &[(usize, Option<usize>, &str)]) -> Spec {
        Spec {
            states: states
                .iter()
                .map(|s| StateSpec {
                    name: s.to_string(),
                    tags: String::new(),
                })
                .collect(),
            initial: 0,
            edges: edges
                .iter()
                .map(|&(from, to, ev)| EdgeSpec {
                    from,
                    to,
                    lines: vec![ev.to_string()],
                })
                .collect(),
        }
    }

    fn overlap(a: &Rect, b: &Rect) -> bool {
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    #[test]
    fn ranks_follow_the_walk_and_unreached_states_start_at_the_top() {
        let s = spec(
            &["A", "B", "C", "D"],
            &[(0, Some(1), "go"), (1, Some(2), "go"), (2, Some(0), "back")],
        );
        assert_eq!(ranks(&s), [0, 1, 2, 0]);
    }

    #[test]
    fn rows_go_down_and_boxes_never_overlap() {
        let s = spec(
            &["A", "B", "C", "D", "E"],
            &[
                (0, Some(1), "next"),
                (0, Some(2), "skip"),
                (1, Some(2), "next"),
                (2, Some(0), "again"),
                (2, Some(3), "done"),
                (3, None, "stay"),
                (1, Some(4), "side"),
                (4, Some(1), "back"),
            ],
        );
        let l = layout(&s);
        let b = &l.boxes;
        assert!(b[0].y < b[1].y && b[1].y == b[2].y && b[2].y < b[3].y);
        for i in 0..b.len() {
            for j in i + 1..b.len() {
                assert!(!overlap(&b[i], &b[j]), "{i} and {j} overlap: {b:?}");
            }
            assert!(b[i].x >= 0.0 && b[i].x + b[i].w <= l.width);
            assert!(b[i].y >= 0.0 && b[i].y + b[i].h <= l.height);
        }
        assert_eq!(l.routes.len(), 8);
        // A loop stays at the right of its box.
        assert!(l.routes[5].label.0 > b[3].x + b[3].w);
        assert!(l.start.1 < b[0].y);
        assert!(l.routes.iter().all(|r| r.d.starts_with('M')));
    }

    #[test]
    fn a_same_row_edge_dips_into_the_label_row() {
        let s = spec(
            &["A", "B", "C"],
            &[(0, Some(1), "x"), (0, Some(2), "y"), (1, Some(2), "side")],
        );
        let l = layout(&s);
        assert_eq!(l.boxes[1].y, l.boxes[2].y);
        assert!(l.routes[2].label.1 > l.boxes[1].y + l.boxes[1].h);
    }

    #[test]
    fn the_layout_is_deterministic() {
        let s = spec(&["A", "B"], &[(0, Some(1), "go"), (1, Some(0), "back")]);
        assert_eq!(layout(&s), layout(&s));
    }

    /// The y of each sideways run of an SVG path of `M`, `L`, and `Q` commands.
    fn sideways(d: &str) -> Vec<f64> {
        let mut out = Vec::new();
        let mut at = (0.0, 0.0);
        for cmd in d.split(['M', 'L', 'Q']) {
            let n: Vec<f64> = cmd
                .split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect();
            if n.len() >= 2 {
                let end = (n[n.len() - 2], n[n.len() - 1]);
                if n.len() == 2 && (end.1 - at.1) == 0.0 && end.0 != at.0 {
                    out.push(end.1);
                }
                at = end;
            }
        }
        out
    }

    #[test]
    fn parallel_edges_take_lanes_of_their_own() {
        let s = spec(
            &["A", "B", "C", "D"],
            &[
                (0, Some(1), "b"),
                (0, Some(2), "c"),
                (0, Some(3), "d"),
                (1, Some(0), "back"),
                (2, Some(0), "back"),
                (3, Some(0), "back"),
            ],
        );
        let l = layout(&s);
        let mut ys: Vec<f64> = l.routes.iter().flat_map(|r| sideways(&r.d)).collect();
        let n = ys.len();
        assert!(n >= 4, "{n} sideways runs");
        ys.sort_by(f64::total_cmp);
        ys.dedup();
        assert_eq!(ys.len(), n, "two runs share a lane");
    }
}
