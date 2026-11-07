//! The edges between two states of one row: which pairs of states the row
//! keeps side by side, the stack of straight lines in the gap between each
//! pair, and the order of a row that keeps each pair together (CLI-8).
//!
//! The states of a chain of pairs, such as `A`–`B`–`C`, sit next to one
//! another with a gap node between each two, so that nothing else in the row
//! comes between them. A pair that would give a state a third neighbour in
//! the row, or close a ring, can't sit side by side, and its edges take the
//! row of labels below, as an edge between two rows does.

use crate::picture::layout::{EdgeSpec, LABEL_GAP, Spec, label_size};

/// The space between a label and the line under it.
const ABOVE: f64 = 2.0;
/// The space between a line and the label of the next line.
const BELOW: f64 = 5.0;
/// The part of a box's side, at its top and its bottom, that a line keeps
/// clear of, so that a line meets the side below the rounded corner.
const CORNER: f64 = 8.0;

/// An edge that crosses the gap between two states of one row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Side {
    /// The gap node between the two states.
    pub gap: usize,
    /// The y of the line, from the centre of the row.
    pub line: f64,
    /// The y of the top of the label, from the centre of the row.
    pub label: f64,
}

/// Two states of one row that the row keeps side by side, with the edges
/// between them, in the order of [`Spec::edges`].
pub(crate) struct Pair {
    pub a: usize,
    pub b: usize,
    pub edges: Vec<usize>,
}

/// The pairs of states that sit side by side: of the pairs of states of one
/// row that edges join, those with the most edges first, and of those with
/// as many, the pair of the earlier first edge, while neither state has two
/// neighbours already and the pair closes no ring. The result is in the
/// order of the pairs' first edges.
// sokf-implements: CLI-8_AC-15
pub(crate) fn pairs(spec: &Spec, rank: &[usize]) -> Vec<Pair> {
    let n = spec.states.len();
    let mut candidates: Vec<Pair> = Vec::new();
    for (k, e) in spec.edges.iter().enumerate() {
        let Some(to) = e.to else { continue };
        if rank[e.from] != rank[to] {
            continue;
        }
        let (a, b) = (e.from.min(to), e.from.max(to));
        match candidates.iter_mut().find(|p| (p.a, p.b) == (a, b)) {
            Some(p) => p.edges.push(k),
            None => candidates.push(Pair {
                a,
                b,
                edges: vec![k],
            }),
        }
    }
    // A stable sort keeps the pairs of as many edges in first-edge order.
    candidates.sort_by_key(|p| std::cmp::Reverse(p.edges.len()));
    let mut degree = vec![0usize; n];
    let mut parent: Vec<usize> = (0..n).collect();
    fn root(parent: &mut [usize], mut v: usize) -> usize {
        while parent[v] != v {
            parent[v] = parent[parent[v]];
            v = parent[v];
        }
        v
    }
    let mut out: Vec<Pair> = Vec::new();
    for p in candidates {
        let (ra, rb) = (root(&mut parent, p.a), root(&mut parent, p.b));
        if degree[p.a] >= 2 || degree[p.b] >= 2 || ra == rb {
            continue;
        }
        parent[ra] = rb;
        degree[p.a] += 1;
        degree[p.b] += 1;
        out.push(p);
    }
    out.sort_by_key(|p| p.edges[0]);
    out
}

/// The lines of a pair's edges in the gap between the two states.
pub(crate) struct Stack {
    /// The offset of each line, and of the top of its label, from the
    /// middle of the row.
    pub lines: Vec<(f64, f64)>,
    /// The width of the widest label.
    pub width: f64,
    /// The height of the stack, the same above and below the middle.
    pub height: f64,
}

/// The lines of a pair's edges, each with its label above it, centred on the
/// middle of the row.
pub(crate) fn stack(edges: &[&EdgeSpec]) -> Stack {
    let mut lines: Vec<(f64, f64)> = Vec::with_capacity(edges.len());
    let mut y = 0.0;
    let mut width: f64 = 0.0;
    for (i, e) in edges.iter().enumerate() {
        let (w, h) = label_size(&e.lines);
        width = width.max(w);
        if i > 0 {
            y += h + ABOVE + BELOW;
        }
        lines.push((y, y - ABOVE - h));
    }
    let shift = y / 2.0;
    let mut reach: f64 = 0.0;
    for l in &mut lines {
        l.0 -= shift;
        l.1 -= shift;
        reach = reach.max(l.0.abs()).max(l.1.abs());
    }
    Stack {
        lines,
        width,
        height: 2.0 * reach,
    }
}

/// The least height of a box whose side the lines at offsets `lines` meet,
/// and whose loops, `loops` high, sit at its top right.
pub(crate) fn least_height(lines: &[f64], loops: f64) -> f64 {
    let mut h: f64 = 0.0;
    for &y in lines {
        h = h.max(2.0 * (y.abs() + CORNER));
        if loops > 0.0 {
            // The loops end above the highest line.
            h = h.max(2.0 * (loops + 4.0 - y));
        }
    }
    h
}

/// The width of the text of a label, without the space around it.
pub(crate) fn text_width(lines: &[String]) -> f64 {
    label_size(lines).0 - LABEL_GAP - 5.0
}

/// `row` in a new order: each node by `key`, each block of nodes kept
/// together in its order or reversed, as the keys of its ends say, and keyed
/// by the mean of the keys of its members that `keyed` accepts.
pub(crate) fn arrange(
    row: &[usize],
    key: impl Fn(usize) -> f64,
    keyed: impl Fn(usize) -> bool,
    blocks: &[Vec<usize>],
    block_of: &[Option<usize>],
) -> Vec<usize> {
    let mut units: Vec<(f64, usize, Vec<usize>)> = Vec::new();
    let mut seen = vec![false; blocks.len()];
    for (i, &v) in row.iter().enumerate() {
        match block_of[v] {
            Some(b) if seen[b] => {}
            Some(b) => {
                seen[b] = true;
                let mut members = blocks[b].clone();
                let ks: Vec<f64> = members
                    .iter()
                    .filter(|&&u| keyed(u))
                    .map(|&u| key(u))
                    .collect();
                let mean = ks.iter().sum::<f64>() / ks.len().max(1) as f64;
                if let (Some(&first), Some(&last)) = (members.first(), members.last())
                    && key(first) > key(last)
                {
                    members.reverse();
                }
                units.push((mean, i, members));
            }
            None => units.push((key(v), i, vec![v])),
        }
    }
    units.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut out = Vec::with_capacity(row.len());
    for (_, _, members) in units {
        out.extend(members);
    }
    out
}

/// The blocks of state indices that sit side by side: each chain of
/// `pairs`, from one end to the other.
pub(crate) fn chains(n: usize, pairs: &[Pair]) -> Vec<Vec<usize>> {
    let mut next: Vec<Vec<usize>> = vec![Vec::new(); n];
    for p in pairs {
        next[p.a].push(p.b);
        next[p.b].push(p.a);
    }
    let mut done = vec![false; n];
    let mut out = Vec::new();
    for start in 0..n {
        if done[start] || next[start].len() != 1 {
            continue;
        }
        let mut chain = vec![start];
        done[start] = true;
        let mut at = start;
        while let Some(&v) = next[at].iter().find(|&&v| !done[v]) {
            done[v] = true;
            chain.push(v);
            at = v;
        }
        out.push(chain);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::picture::layout::StateSpec;

    fn spec(n: usize, edges: &[(usize, usize)]) -> Spec {
        Spec {
            states: (0..n)
                .map(|i| StateSpec {
                    name: format!("S{i}"),
                    tags: String::new(),
                })
                .collect(),
            initial: 0,
            edges: edges
                .iter()
                .map(|&(from, to)| EdgeSpec {
                    from,
                    to: Some(to),
                    lines: vec!["go".into()],
                })
                .collect(),
        }
    }

    // sokf-tests: CLI-8_AC-15
    #[test]
    fn a_third_neighbour_or_a_ring_keeps_its_pair_apart() {
        // 1, 2, 3, and 4 share a row: 1–2–3 is a chain, 3–1 closes a ring,
        // and 2–4 would give 2 a third neighbour.
        let s = spec(5, &[(0, 1), (1, 2), (2, 1), (2, 3), (3, 1), (1, 4), (0, 4)]);
        let rank = [0, 1, 1, 1, 1];
        let p = pairs(&s, &rank);
        let keys: Vec<(usize, usize, Vec<usize>)> =
            p.iter().map(|p| (p.a, p.b, p.edges.clone())).collect();
        assert_eq!(keys, [(1, 2, vec![1, 2]), (2, 3, vec![3]), (1, 4, vec![5])]);
        assert_eq!(chains(5, &p), [vec![3, 2, 1, 4]]);
    }

    // sokf-tests: CLI-8_AC-15
    #[test]
    fn the_pair_with_the_most_edges_sits_side_by_side_first() {
        // S, 1, has three neighbours in its row: A and B by one edge each,
        // and C, last, by seven.
        let mut edges = vec![(1, 2), (1, 3)];
        edges.extend([(1, 4); 7]);
        let s = spec(5, &edges);
        let p = pairs(&s, &[0, 1, 1, 1, 1]);
        let keys: Vec<(usize, usize, usize)> =
            p.iter().map(|p| (p.a, p.b, p.edges.len())).collect();
        assert_eq!(keys, [(1, 2, 1), (1, 4, 7)]);
    }

    #[test]
    fn a_stack_centres_its_lines_and_puts_each_label_above_its_line() {
        let a = EdgeSpec {
            from: 0,
            to: Some(1),
            lines: vec!["go".into()],
        };
        let one = stack(&[&a]);
        let lines = &one.lines;
        assert_eq!(lines[0].0, 0.0);
        assert!(lines[0].1 < 0.0 && one.height >= 2.0 * -lines[0].1);
        let b = EdgeSpec {
            lines: vec!["back".into(), "[visits A ≥ 2]".into()],
            ..a
        };
        let lines = stack(&[&a, &b]).lines;
        assert_eq!(lines[0].0, -lines[1].0);
        // The second label sits between the two lines.
        assert!(lines[1].1 > lines[0].0 && lines[1].1 + 30.0 < lines[1].0);
        assert!(least_height(&[lines[0].0, lines[1].0], 0.0) > 32.0);
        assert!(least_height(&[0.0], 21.0) >= 50.0);
    }

    #[test]
    fn a_block_moves_as_one_and_turns_to_its_neighbours() {
        let blocks = vec![vec![1, 2, 3]];
        let block_of = [None, Some(0), Some(0), Some(0), None];
        let key = |v: usize| [5.0, 4.0, 0.0, 1.0, 2.0][v];
        let out = arrange(&[0, 4, 1, 2, 3], key, |v| v != 2, &blocks, &block_of);
        assert_eq!(out, [4, 3, 2, 1, 0]);
    }
}
