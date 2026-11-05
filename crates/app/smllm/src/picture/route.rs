//! The routes of a layout's edges: ports spread along each side of a box,
//! a lane of its own for each edge that moves sideways between two rows, and
//! rounded corners of one radius (CLI-8).

use std::fmt::Write as _;

use crate::picture::layout::{
    LABEL_GAP, LOOP_OUT, Layout, MARGIN, Node, Pt, Rect, START, Spec, label_size,
};

/// The space between two lanes, and between a lane and a row.
const LANE: f64 = 7.0;
/// The least space between two rows.
const ROW_GAP: f64 = 16.0;
/// The radius of a corner.
const RADIUS: f64 = 9.0;

/// The height of each row: its tallest node, and at least 18 for a row of
/// labels.
fn row_heights(nodes: &[Node], n_rows: usize) -> Vec<f64> {
    let mut h = vec![0.0_f64; n_rows];
    for n in nodes {
        h[n.row] = h[n.row].max(n.h);
    }
    for (r, h) in h.iter_mut().enumerate() {
        if r % 2 == 1 {
            *h = h.max(18.0);
        }
    }
    h
}

/// A polyline with each corner rounded to [`RADIUS`], or less where a
/// segment is short.
fn rounded(pts: &[Pt]) -> String {
    // Drop repeated points and the middle of three points in a line.
    let mut p: Vec<Pt> = Vec::with_capacity(pts.len());
    for &q in pts {
        if p.last()
            .is_some_and(|l| (l.0 - q.0).abs() < 0.05 && (l.1 - q.1).abs() < 0.05)
        {
            continue;
        }
        if p.len() >= 2 {
            let (a, b) = (p[p.len() - 2], p[p.len() - 1]);
            let cross = (b.0 - a.0) * (q.1 - b.1) - (b.1 - a.1) * (q.0 - b.0);
            let dot = (b.0 - a.0) * (q.0 - b.0) + (b.1 - a.1) * (q.1 - b.1);
            if cross.abs() < 0.01 && dot > 0.0 {
                p.pop();
            }
        }
        p.push(q);
    }
    let mut d = format!("M{:.1} {:.1}", p[0].0, p[0].1);
    for i in 1..p.len() {
        let b = p[i];
        if i + 1 == p.len() {
            let _ = write!(d, "L{:.1} {:.1}", b.0, b.1);
            break;
        }
        let (a, c) = (p[i - 1], p[i + 1]);
        let len = |u: Pt, v: Pt| (v.0 - u.0).hypot(v.1 - u.1);
        let (l1, l2) = (len(a, b), len(b, c));
        let r = RADIUS.min(l1 / 2.0).min(l2 / 2.0);
        let s = (b.0 + (a.0 - b.0) * r / l1, b.1 + (a.1 - b.1) * r / l1);
        let e = (b.0 + (c.0 - b.0) * r / l2, b.1 + (c.1 - b.1) * r / l2);
        let _ = write!(
            d,
            "L{:.1} {:.1}Q{:.1} {:.1} {:.1} {:.1}",
            s.0, s.1, b.0, b.1, e.0, e.1
        );
    }
    d
}

/// One sideways step of a route between two rows: its path, its step in the
/// path, the row above it, and its ends above and below.
struct Jog {
    path: usize,
    step: usize,
    row: usize,
    upper: f64,
    lower: f64,
}

/// The geometry of a layout: the boxes, the ports, the lanes, and the routes.
pub(crate) fn geometry(
    spec: &Spec,
    nodes: &[Node],
    n_rows: usize,
    state_node: &[usize],
    paths: &[Vec<usize>],
    edge_path: &[Option<usize>],
) -> Layout {
    let heights = row_heights(nodes, n_rows);
    // Spread the ends of the edges along each side of a box, in the order of
    // the points that they come from.
    let mut ends: Vec<Vec<(usize, bool, bool, f64)>> = vec![Vec::new(); nodes.len()];
    for (p, path) in paths.iter().enumerate() {
        let last = path.len() - 1;
        for (end, next) in [(0, 1), (last, last - 1)] {
            let (v, u) = (path[end], path[next]);
            let top = nodes[u].row < nodes[v].row;
            ends[v].push((p, end == 0, top, nodes[u].x));
        }
    }
    // The x of each end of each path, and whether it is on the top of its box.
    let mut port = vec![[(0.0, false); 2]; paths.len()];
    for (v, list) in ends.iter().enumerate() {
        let (x0, w) = (nodes[v].x - nodes[v].left, 2.0 * nodes[v].left);
        for top in [true, false] {
            let mut side: Vec<&(usize, bool, bool, f64)> =
                list.iter().filter(|e| e.2 == top).collect();
            side.sort_by(|a, b| a.3.total_cmp(&b.3).then(a.0.cmp(&b.0)));
            let n = side.len() as f64;
            for (i, &&(p, first, _, _)) in side.iter().enumerate() {
                let x = if side.len() == 1 {
                    nodes[v].x
                } else {
                    x0 + w * (0.1 + 0.8 * (i as f64 + 0.5) / n)
                };
                port[p][usize::from(!first)] = (x, top);
            }
        }
    }
    // A label that a route enters and leaves on the same side: the label of
    // an edge between two states of one row, which the route wraps.
    let dip = |p: usize, i: usize| {
        let path = &paths[p];
        let n = &nodes[path[i]];
        i > 0
            && i + 1 < path.len()
            && n.h > 0.0
            && (nodes[path[i - 1]].row < n.row) != (nodes[path[i + 1]].row > n.row)
    };
    // The x where a route enters a node, or with `leaving` where it leaves it.
    let x_at = |p: usize, i: usize, leaving: bool| {
        let path = &paths[p];
        if i == 0 {
            port[p][0].0
        } else if i == path.len() - 1 {
            port[p][1].0
        } else if leaving && dip(p, i) {
            nodes[path[i]].x + nodes[path[i]].right
        } else {
            nodes[path[i]].x
        }
    };
    // Each sideways step gets a lane of its own in the gap below its upper row,
    // ordered so that steps in one direction cross each other least.
    let mut jogs: Vec<Jog> = Vec::new();
    for (p, path) in paths.iter().enumerate() {
        for i in 0..path.len() - 1 {
            let (a, b) = (path[i], path[i + 1]);
            let (xa, xb) = (x_at(p, i, true), x_at(p, i + 1, false));
            if (xa - xb).abs() < 0.5 {
                continue;
            }
            let down = nodes[a].row < nodes[b].row;
            let (upper, lower) = if down { (xa, xb) } else { (xb, xa) };
            jogs.push(Jog {
                path: p,
                step: i,
                row: nodes[a].row.min(nodes[b].row),
                upper,
                lower,
            });
        }
    }
    let key = |j: &Jog| {
        if j.lower > j.upper {
            (0, -j.upper)
        } else {
            (1, j.upper)
        }
    };
    jogs.sort_by(|a, b| {
        a.row
            .cmp(&b.row)
            .then(key(a).0.cmp(&key(b).0))
            .then(key(a).1.total_cmp(&key(b).1))
    });
    let mut count = vec![0usize; n_rows];
    let mut lane = std::collections::HashMap::new();
    for j in &jogs {
        lane.insert((j.path, j.step), count[j.row]);
        count[j.row] += 1;
    }
    // The rows from the top, each gap tall enough for its lanes.
    let mut ys = Vec::with_capacity(n_rows);
    let mut y = MARGIN;
    for r in 0..n_rows {
        ys.push(y + heights[r] / 2.0);
        y += heights[r];
        if r + 1 < n_rows {
            y += ROW_GAP.max(LANE * (count[r] as f64 + 1.0));
        }
    }
    let height = y + MARGIN;
    let rect = |v: usize| {
        let n = &nodes[v];
        Rect {
            x: n.x - n.left,
            y: ys[n.row] - n.h / 2.0,
            w: 2.0 * n.left,
            h: n.h,
        }
    };
    let boxes: Vec<Rect> = state_node.iter().map(|&v| rect(v)).collect();
    let lane_y = |row: usize, k: usize| {
        let lo = ys[row] + heights[row] / 2.0;
        let hi = ys[row + 1] - heights[row + 1] / 2.0;
        lo + (hi - lo) * (k as f64 + 1.0) / (count[row] as f64 + 1.0)
    };
    let polyline = |p: usize| {
        let path = &paths[p];
        let last = path.len() - 1;
        // Each node of the path as the points of the route through it: a port
        // on its box, the left side of a label, or a turn around a label.
        let mut stops: Vec<Vec<Pt>> = Vec::new();
        for i in 0..=last {
            let n = &nodes[path[i]];
            let x = x_at(p, i, false);
            if i == 0 || i == last {
                let r = rect(path[i]);
                let top = port[p][usize::from(i == last)].1;
                let y = if top { r.y } else { r.y + r.h };
                stops.push(vec![(x, y)]);
                continue;
            }
            let (y, half) = (ys[n.row], n.h / 2.0);
            let down_in = nodes[path[i - 1]].row < n.row;
            let enter = if down_in { y - half } else { y + half };
            if dip(p, i) {
                let far = if down_in {
                    y + half + 3.0
                } else {
                    y - half - 3.0
                };
                let right = x_at(p, i, true);
                stops.push(vec![(x, enter), (x, far), (right, far), (right, enter)]);
            } else if half > 0.0 {
                let leave = if down_in { y + half } else { y - half };
                stops.push(vec![(x, enter), (x, leave)]);
            } else {
                stops.push(vec![(x, enter)]);
            }
        }
        let mut pts = stops[0].clone();
        for i in 1..=last {
            let from = *pts.last().unwrap_or(&stops[0][0]);
            let to = stops[i][0];
            if let Some(&k) = lane.get(&(p, i - 1)) {
                let row = nodes[path[i - 1]].row.min(nodes[path[i]].row);
                let ly = lane_y(row, k);
                pts.push((from.0, ly));
                pts.push((to.0, ly));
            }
            pts.extend(stops[i].iter().copied());
        }
        pts
    };
    let mut loops = vec![0.0; spec.states.len()];
    let routes = spec
        .edges
        .iter()
        .enumerate()
        .map(|(k, e)| match edge_path[k] {
            Some(p) => {
                let n = &nodes[paths[p][1]];
                crate::picture::layout::Route {
                    d: rounded(&polyline(p)),
                    label: (n.x + LABEL_GAP, ys[n.row] - n.h / 2.0),
                }
            }
            None => {
                let b = boxes[e.from];
                let (_, lh) = label_size(&e.lines);
                let top = b.y + 3.0 + loops[e.from];
                loops[e.from] += lh + 6.0;
                let (x, y0, y1) = (b.x + b.w, top + 2.0, top + lh + 2.0);
                let out = x + LOOP_OUT * 0.8;
                crate::picture::layout::Route {
                    d: format!("M{x:.1} {y0:.1}C{out:.1} {y0:.1} {out:.1} {y1:.1} {x:.1} {y1:.1}"),
                    label: (x + LOOP_OUT, top + 2.0),
                }
            }
        })
        .collect();
    let width = nodes.iter().map(|n| n.x + n.right).fold(0.0, f64::max) + MARGIN;
    let s = &nodes[0];
    let mut start = polyline(0);
    start[0] = (s.x, ys[0] + START / 2.0);
    Layout {
        width,
        height,
        start: (s.x, ys[0]),
        start_route: rounded(&start),
        boxes,
        routes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_round_and_straight_runs_merge() {
        assert_eq!(
            rounded(&[(0.0, 0.0), (0.0, 5.0), (0.0, 10.0)]),
            "M0.0 0.0L0.0 10.0"
        );
        let d = rounded(&[(0.0, 0.0), (0.0, 20.0), (30.0, 20.0), (30.0, 40.0)]);
        assert_eq!(
            d,
            "M0.0 0.0L0.0 11.0Q0.0 20.0 9.0 20.0L21.0 20.0Q30.0 20.0 30.0 29.0L30.0 40.0"
        );
        // A short step rounds less.
        let d = rounded(&[(0.0, 0.0), (0.0, 4.0), (6.0, 4.0), (6.0, 8.0)]);
        assert!(
            d.contains("Q0.0 4.0 2.0 4.0") || d.contains("Q0.0 4.0 3.0 4.0"),
            "{d}"
        );
    }
}
