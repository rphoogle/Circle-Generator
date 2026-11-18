//! Shape layout: works out WHERE every brick goes, as flat rectangles and
//! right-angled triangles seen from above. Nothing here knows about .brz files.
//!
//! Coordinates are "half-cells" (h): one grid cell (a micro brick, or a stud
//! in stud mode) is 2 h wide. Using half-cells lets odd diameters put the
//! circle's centre in the middle of a cell while every brick edge stays on
//! the grid. The circle's centre is always (0, 0).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Full,
    Half,
    Quarter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Solid,
    Hollow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// Parallel bars (Solid and Solid inverse)
    Strips,
    /// Square centre (Solid) / Corner blocks (Solid inverse)
    Blocks,
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub shape: Shape,
    pub kind: Kind,
    pub inverse: bool,
    pub fill: Fill,
    /// Diameter in grid cells (micro bricks, or studs)
    pub diameter: i32,
    /// Wall thickness in cells at the thinnest point (Solid inverse only)
    pub wall: i32,
}

/// One piece seen from above, in half-cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// Filler brick covering x0..x1, y0..y1
    Rect { x0: i32, y0: i32, x1: i32, y1: i32 },
    /// Wedge filling the triangle inside the box x0..x1, y0..y1 whose right
    /// angle sits at `corner`: 0 = (-x,-y), 1 = (+x,-y), 2 = (+x,+y), 3 = (-x,+y).
    /// Corner 0 is the wedge's own unrotated shape; corner c = c quarter-turns
    /// anticlockwise (confirmed in game).
    Tri { x0: i32, y0: i32, x1: i32, y1: i32, corner: u8 },
}

impl Piece {
    pub fn bbox(&self) -> (i32, i32, i32, i32) {
        match *self {
            Piece::Rect { x0, y0, x1, y1 } | Piece::Tri { x0, y0, x1, y1, .. } => (x0, y0, x1, y1),
        }
    }
    pub fn is_wedge(&self) -> bool {
        matches!(self, Piece::Tri { .. })
    }
    /// Outline points (for drawing / checking)
    pub fn polygon(&self) -> Vec<(i32, i32)> {
        match *self {
            Piece::Rect { x0, y0, x1, y1 } => vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)],
            Piece::Tri { x0, y0, x1, y1, corner } => {
                let c = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
                let k = corner as usize;
                vec![c[k], c[(k + 1) % 4], c[(k + 3) % 4]]
            }
        }
    }
}

fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Piece {
    Piece::Rect { x0: x0.min(x1), y0: y0.min(y1), x1: x0.max(x1), y1: y0.max(y1) }
}

/// Smoothest convex outline on the grid for the top-right quarter, from the
/// +x axis point (D, 0) round to the +y axis point (0, D), in half-cells.
/// Every vertex except the two axis points is a real grid point.
pub fn arc(diameter: i32) -> Vec<(i32, i32)> {
    let d = diameter; // radius in half-cells
    let p = d.rem_euclid(2); // grid points sit on this parity
    let lim = d as f64 + 0.6; // 0.3 cell outside the true circle
    // top-most grid point in each column
    let mut tops = vec![];
    let mut x = p;
    while x <= d {
        let rem = lim * lim - (x as f64) * (x as f64);
        if rem >= 0.0 {
            let mut y = rem.sqrt().floor() as i32;
            if y > d {
                y = d;
            }
            if (y - p).rem_euclid(2) != 0 {
                y -= 1;
            }
            if y >= p {
                tops.push((x, y));
            }
        }
        x += 2;
    }
    // upper convex hull, left to right
    let cross = |o: (i32, i32), a: (i32, i32), b: (i32, i32)| {
        (a.0 - o.0) as i64 * (b.1 - o.1) as i64 - (a.1 - o.1) as i64 * (b.0 - o.0) as i64
    };
    let mut hull: Vec<(i32, i32)> = vec![];
    for &pt in &tops {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], pt) >= 0 {
            hull.pop();
        }
        hull.push(pt);
    }
    hull.reverse(); // now from the +x side round to the +y side
    if hull[0] != (d, 0) {
        hull.insert(0, (d, 0));
    }
    if *hull.last().unwrap() != (0, d) {
        hull.push((0, d));
    }
    hull
}

type Seg = ((i32, i32), (i32, i32));
fn segs(a: &[(i32, i32)]) -> Vec<Seg> {
    a.windows(2).map(|w| (w[0], w[1])).collect()
}

/// Quarters in play: 1 = +x+y, 2 = -x+y, 3 = -x-y, 4 = +x-y
fn quads(shape: Shape) -> [bool; 5] {
    match shape {
        Shape::Full => [false, true, true, true, true],
        Shape::Half => [false, true, true, false, false],
        Shape::Quarter => [false, true, false, false, false],
    }
}

/// Copy quarter-1 pieces into the other quarters that are in play.
fn mirror(q1: &[Piece], q: &[bool; 5]) -> Vec<Piece> {
    let mut out = vec![];
    for (qi, (sx, sy)) in [(1, (1, 1)), (2, (-1, 1)), (3, (-1, -1)), (4, (1, -1))] {
        if !q[qi] {
            continue;
        }
        for p in q1 {
            out.push(match *p {
                Piece::Rect { x0, y0, x1, y1 } => rect(sx * x0, sy * y0, sx * x1, sy * y1),
                Piece::Tri { x0, y0, x1, y1, corner } => {
                    let (cx, cy) = [(-1, -1), (1, -1), (1, 1), (-1, 1)][corner as usize];
                    let (cx, cy) = (cx * sx, cy * sy);
                    let corner = match (cx, cy) {
                        (-1, -1) => 0,
                        (1, -1) => 1,
                        (1, 1) => 2,
                        _ => 3,
                    };
                    let (a0, a1) = ((sx * x0).min(sx * x1), (sx * x0).max(sx * x1));
                    let (b0, b1) = ((sy * y0).min(sy * y1), (sy * y0).max(sy * y1));
                    Piece::Tri { x0: a0, y0: b0, x1: a1, y1: b1, corner }
                }
            });
        }
    }
    out
}

/// A rectangle that may run across one or both centre lines: clip it to the
/// quarters in play. `xs`/`ys` say which side(s) of the centre it covers.
fn span_rect(q: &[bool; 5], x_lo: i32, x_hi: i32, y_lo: i32, y_hi: i32, out: &mut Vec<Piece>) {
    // Each of x_lo..x_hi / y_lo..y_hi may straddle 0; keep only the parts in play.
    let (x_lo, x_hi) = (x_lo.min(x_hi), x_lo.max(x_hi));
    let (y_lo, y_hi) = (y_lo.min(y_hi), y_lo.max(y_hi));
    let left = q[2] || q[3];
    let right = q[1] || q[4];
    let up = q[1] || q[2];
    let down = q[3] || q[4];
    let xa = if left { x_lo } else { x_lo.max(0) };
    let xb = if right { x_hi } else { x_hi.min(0) };
    let ya = if down { y_lo } else { y_lo.max(0) };
    let yb = if up { y_hi } else { y_hi.min(0) };
    if xa < xb && ya < yb {
        out.push(rect(xa, ya, xb, yb));
    }
}

fn outer_wedges(a: &[(i32, i32)]) -> Vec<Piece> {
    segs(a)
        .into_iter()
        .filter(|((x0, y0), (x1, y1))| x0 > x1 && y1 > y0)
        .map(|((x0, y0), (x1, y1))| Piece::Tri { x0: x1, y0, x1: x0, y1, corner: 0 })
        .collect()
}

fn inner_wedges(a: &[(i32, i32)]) -> Vec<Piece> {
    segs(a)
        .into_iter()
        .filter(|((x0, y0), (x1, y1))| x0 > x1 && y1 > y0)
        .map(|((x0, y0), (x1, y1))| Piece::Tri { x0: x1, y0, x1: x0, y1, corner: 2 })
        .collect()
}

/// Where the square centre's corner goes: (k, k). Also returns the corner of
/// the stepped outline used by Corner blocks.
fn diagonal(a: &[(i32, i32)]) -> ((i32, i32), (i32, i32)) {
    // a vertex exactly on the diagonal?
    if let Some(&(v, _)) = a.iter().find(|(x, y)| x == y) {
        return ((v, v), (v, v));
    }
    for ((x0, y0), (x1, y1)) in segs(a) {
        if x0 > y0 && x1 < y1 {
            return ((x1, y0), (x0, y1));
        }
    }
    unreachable!("outline always crosses the diagonal")
}

pub fn layout(l: &Layout) -> Vec<Piece> {
    let d = l.diameter;
    let a = arc(d);
    let q = quads(l.shape);
    let mut out = vec![];
    match (l.kind, l.inverse) {
        // ---------------- solid disc ----------------
        (Kind::Solid, false) => {
            out.extend(mirror(&outer_wedges(&a), &q));
            match l.fill {
                Fill::Strips => {
                    for ((x0, y0), (x1, _)) in segs(&a) {
                        if x0 > x1 && y0 > 0 {
                            span_rect(&q, x1, x0, -y0, y0, &mut out);
                            span_rect(&q, -x0, -x1, -y0, y0, &mut out);
                        }
                    }
                }
                Fill::Blocks => {
                    let ((k, _), _) = diagonal(&a);
                    span_rect(&q, -k, k, -k, k, &mut out);
                    for ((x0, y0), (x1, _)) in segs(&a) {
                        if x0 > x1 && x1 >= k && y0 > 0 {
                            span_rect(&q, x1, x0, -y0, y0, &mut out); // +x side
                            span_rect(&q, -x0, -x1, -y0, y0, &mut out); // -x side
                            span_rect(&q, -y0, y0, x1, x0, &mut out); // +y side
                            span_rect(&q, -y0, y0, -x0, -x1, &mut out); // -y side
                        }
                    }
                }
            }
        }
        // ---------------- hollow ring (outer edge only) ----------------
        (Kind::Hollow, false) => {
            out.extend(mirror(&outer_wedges(&a), &q));
            let (ax, ay) = a[1];
            if ax == d && ay > 0 {
                let bx = a[2].0;
                span_rect(&q, bx, d, -ay, ay, &mut out);
                span_rect(&q, -d, -bx, -ay, ay, &mut out);
                span_rect(&q, -ay, ay, bx, d, &mut out);
                span_rect(&q, -ay, ay, -d, -bx, &mut out);
            }
        }
        // ---------------- hollow inverse (smooth hole, wedges only) ----------------
        (Kind::Hollow, true) => {
            out.extend(mirror(&inner_wedges(&a), &q));
            let (ax, ay) = a[1];
            if ax == d && ay > 0 {
                let t = d + 2; // 1-cell bar behind each straight part
                span_rect(&q, d, t, -ay, ay, &mut out);
                span_rect(&q, -t, -d, -ay, ay, &mut out);
                span_rect(&q, -ay, ay, d, t, &mut out);
                span_rect(&q, -ay, ay, -t, -d, &mut out);
            }
        }
        // ---------------- solid inverse (block with a round hole) ----------------
        (Kind::Solid, true) => {
            let s = d + 2 * l.wall;
            out.extend(mirror(&inner_wedges(&a), &q));
            match l.fill {
                Fill::Strips => {
                    span_rect(&q, d, s, -s, s, &mut out);
                    span_rect(&q, -s, -d, -s, s, &mut out);
                    for ((x0, _), (x1, y1)) in segs(&a) {
                        if x0 > x1 {
                            span_rect(&q, x1, x0, y1, s, &mut out);
                            span_rect(&q, x1, x0, -s, -y1, &mut out);
                            span_rect(&q, -x0, -x1, y1, s, &mut out);
                            span_rect(&q, -x0, -x1, -s, -y1, &mut out);
                        }
                    }
                }
                Fill::Blocks => {
                    let (_, (kx, ky)) = diagonal(&a);
                    span_rect(&q, kx, s, ky, s, &mut out);
                    span_rect(&q, -s, -kx, ky, s, &mut out);
                    span_rect(&q, -s, -kx, -s, -ky, &mut out);
                    span_rect(&q, kx, s, -s, -ky, &mut out);
                    for ((x0, y0), (_, y1)) in segs(&a) {
                        if y1 > y0 && y1 <= ky {
                            // +x and -x sides (bars from the steps out to the wall)
                            if y0 == 0 {
                                span_rect(&q, x0, s, -y1, y1, &mut out);
                                span_rect(&q, -s, -x0, -y1, y1, &mut out);
                                span_rect(&q, -y1, y1, x0, s, &mut out);
                                span_rect(&q, -y1, y1, -s, -x0, &mut out);
                            } else {
                                for (sx, sy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                                    span_rect(&q, sx * x0, sx * s, sy * y0, sy * y1, &mut out);
                                    span_rect(&q, sx * y0, sx * y1, sy * x0, sy * s, &mut out);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    merge(out)
}

/// Join filler pieces that sit edge to edge with exactly the same span
/// (e.g. the two halves of a centre strip either side of the centre line).
fn merge(mut ps: Vec<Piece>) -> Vec<Piece> {
    loop {
        let mut done = true;
        'outer: for i in 0..ps.len() {
            for j in 0..ps.len() {
                if i == j {
                    continue;
                }
                if let (Piece::Rect { x0: a0, y0: b0, x1: a1, y1: b1 }, Piece::Rect { x0: c0, y0: e0, x1: c1, y1: e1 }) =
                    (ps[i], ps[j])
                {
                    // the halves are only merged if one of them is too thin to be a real brick on its own
                    let thin = |w: i32| w % 2 != 0;
                    if a1 == c0 && b0 == e0 && b1 == e1 && (thin(a1 - a0) || thin(c1 - c0) || a0 == -c1) {
                        ps[i] = rect(a0, b0, c1, b1);
                        ps.remove(j);
                        done = false;
                        break 'outer;
                    }
                    if b1 == e0 && a0 == c0 && a1 == c1 && (thin(b1 - b0) || thin(e1 - e0) || b0 == -e1) {
                        ps[i] = rect(a0, b0, a1, e1);
                        ps.remove(j);
                        done = false;
                        break 'outer;
                    }
                }
            }
        }
        if done {
            return ps;
        }
    }
}
