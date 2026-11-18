//! Shape layout: works out WHERE every brick goes, as flat rectangles and
//! right-angled triangles seen from above. Nothing here knows about .brz files.
//!
//! Coordinates are "half-cells" (h): one grid cell (a micro brick, or a stud
//! in stud mode) is 2 h wide. Using half-cells lets odd sizes put the centre
//! in the middle of a cell while every brick edge stays on the grid. The
//! shape's centre is always (0, 0). Circles are simply ovals whose width and
//! length are equal.

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
    /// Doughnut: a filled band between an outer and an inner edge
    Ring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// Parallel bars
    Strips,
    /// Square/rectangle centre (Solid), corner blocks (Solid inverse),
    /// four-way strips (Doughnut)
    Blocks,
}

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub shape: Shape,
    pub kind: Kind,
    pub inverse: bool,
    pub fill: Fill,
    /// Outer width (x) and length (y) in grid cells
    pub width: i32,
    pub length: i32,
    /// Doughnut hole width / length in cells (Ring only)
    pub hole_w: i32,
    pub hole_l: i32,
    /// Wall thickness in cells at the thinnest point (inverse shapes)
    pub wall: i32,
}

impl Layout {
    /// A plain circle (width = length)
    pub fn circle(shape: Shape, kind: Kind, inverse: bool, fill: Fill, diameter: i32, wall: i32) -> Self {
        Layout { shape, kind, inverse, fill, width: diameter, length: diameter, hole_w: 0, hole_l: 0, wall }
    }
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

/// Smoothest convex outline on the grid for the top-right quarter of an oval
/// `w` cells wide and `l` cells long, from the +x axis point (w, 0) round to
/// the +y axis point (0, l), in half-cells. Every vertex except the two axis
/// points is a real grid point.
pub fn arc(w: i32, l: i32) -> Vec<(i32, i32)> {
    let (a, b) = (w, l); // half-axes in half-cells
    let (px, py) = (a.rem_euclid(2), b.rem_euclid(2)); // grid points sit on these parities
    let (ea, eb) = (a as f64 + 0.6, b as f64 + 0.6); // 0.3 cell outside the true outline
    let mut tops = vec![];
    let mut x = px;
    while x <= a {
        let t = 1.0 - (x as f64 / ea).powi(2);
        if t >= 0.0 {
            let mut y = ((eb * t.sqrt()).floor() as i32).min(b);
            if (y - py).rem_euclid(2) != 0 {
                y -= 1;
            }
            if y >= py {
                tops.push((x, y));
            }
        }
        x += 2;
    }
    // the extreme columns always reach the full size, so the outline meets
    // the bounding box exactly and its ends stay on the grid
    if let Some(t) = tops.first_mut() {
        if t.0 == px {
            t.1 = b;
        }
    }
    match tops.last_mut() {
        Some(t) if t.0 == a => t.1 = t.1.max(py),
        _ => tops.push((a, py)),
    }
    let mut hull: Vec<(i32, i32)> = vec![];
    for &pt in &tops {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], pt) >= 0 {
            hull.pop();
        }
        hull.push(pt);
    }
    hull.reverse(); // now from the +x side round to the +y side
    if hull[0] != (a, 0) {
        hull.insert(0, (a, 0));
    }
    if *hull.last().unwrap() != (0, b) {
        hull.push((0, b));
    }
    hull
}

fn cross(o: (i32, i32), a: (i32, i32), b: (i32, i32)) -> i64 {
    (a.0 - o.0) as i64 * (b.1 - o.1) as i64 - (a.1 - o.1) as i64 * (b.0 - o.0) as i64
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
                    let corner = match (cx * sx, cy * sy) {
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
/// quarters in play.
fn span_rect(q: &[bool; 5], x_lo: i32, x_hi: i32, y_lo: i32, y_hi: i32, out: &mut Vec<Piece>) {
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

/// Same rectangle in all four mirror positions (clipped to the quarters in play).
fn span4(q: &[bool; 5], x0: i32, x1: i32, y0: i32, y1: i32, out: &mut Vec<Piece>) {
    for (sx, sy) in [(1, 1), (-1, 1), (-1, -1), (1, -1)] {
        span_rect(q, sx * x0, sx * x1, sy * y0, sy * y1, out);
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

/// Where the outline crosses the line from the centre to the corner of its
/// bounding box (the diagonal, for a circle). Returns the corner for the
/// centre square/rectangle, and the corner of the stepped outline used by
/// Corner blocks.
fn diagonal(a: &[(i32, i32)]) -> ((i32, i32), (i32, i32)) {
    let (aa, bb) = (a[0].0 as i64, a.last().unwrap().1 as i64);
    let side = |(x, y): (i32, i32)| x as i64 * bb - y as i64 * aa; // >0 below the diagonal
    if let Some(&p) = a[1..a.len() - 1].iter().find(|&&p| side(p) == 0) {
        return (p, p);
    }
    for ((x0, y0), (x1, y1)) in segs(a) {
        if side((x0, y0)) > 0 && side((x1, y1)) < 0 {
            return ((x1, y0), (x0, y1));
        }
    }
    unreachable!("outline always crosses the diagonal")
}

/// Solid oval (outward wedges + filler).
fn solid(a: &[(i32, i32)], q: &[bool; 5], fill: Fill, out: &mut Vec<Piece>) {
    out.extend(mirror(&outer_wedges(a), q));
    match fill {
        Fill::Strips => {
            for ((x0, y0), (x1, _)) in segs(a) {
                if x0 > x1 && y0 > 0 {
                    span_rect(q, x1, x0, -y0, y0, out);
                    span_rect(q, -x0, -x1, -y0, y0, out);
                }
            }
        }
        Fill::Blocks => {
            let ((kx, ky), _) = diagonal(a);
            span_rect(q, -kx, kx, -ky, ky, out);
            for ((x0, y0), (x1, y1)) in segs(a) {
                if x0 > x1 && x1 >= kx && y0 > 0 {
                    span_rect(q, x1, x0, -y0, y0, out); // +x side
                    span_rect(q, -x0, -x1, -y0, y0, out); // -x side
                }
                if y1 > y0 && y0 >= ky && x1 > 0 {
                    span_rect(q, -x1, x1, y0, y1, out); // +y side
                    span_rect(q, -x1, x1, -y1, -y0, out); // -y side
                }
            }
        }
    }
}

/// Block with an oval hole (inward wedges + filler out to the wall).
fn solid_inverse(a: &[(i32, i32)], q: &[bool; 5], fill: Fill, wall: i32, out: &mut Vec<Piece>) {
    let (aa, bb) = (a[0].0, a.last().unwrap().1);
    let (sx, sy) = (aa + 2 * wall, bb + 2 * wall);
    out.extend(mirror(&inner_wedges(a), q));
    match fill {
        Fill::Strips => {
            span_rect(q, aa, sx, -sy, sy, out);
            span_rect(q, -sx, -aa, -sy, sy, out);
            for ((x0, _), (x1, y1)) in segs(a) {
                if x0 > x1 {
                    span4(q, x1, x0, y1, sy, out);
                }
            }
        }
        Fill::Blocks => {
            let (_, (kx, ky)) = diagonal(a);
            span4(q, kx, sx, ky, sy, out);
            for ((x0, y0), (x1, y1)) in segs(a) {
                // +x / -x sides: flat bars from the steps out to the wall
                if y1 > y0 && y1 <= ky {
                    if y0 == 0 {
                        span_rect(q, x0, sx, -y1, y1, out);
                        span_rect(q, -sx, -x0, -y1, y1, out);
                    } else {
                        span4(q, x0, sx, y0, y1, out);
                    }
                }
                // +y / -y sides: upright bars from the steps out to the wall
                if x0 > x1 && x0 <= kx {
                    if x1 == 0 {
                        span_rect(q, -x0, x0, y1, sy, out);
                        span_rect(q, -x0, x0, -sy, -y1, out);
                    } else {
                        span4(q, x1, x0, y1, sy, out);
                    }
                }
            }
        }
    }
}

/// Bars behind the straight bits on each axis (closes a hollow ring).
fn axis_bars(a: &[(i32, i32)], q: &[bool; 5], inverse: bool, out: &mut Vec<Piece>) {
    let (aa, bb) = (a[0].0, a.last().unwrap().1);
    let n = a.len();
    // straight bit on the x axis: from (aa,0) up to (aa,ay)
    let (ax, ay) = a[1];
    if ax == aa && ay > 0 {
        let (lo, hi) = if inverse { (aa, aa + 2) } else { (a[2].0, aa) };
        span_rect(q, lo, hi, -ay, ay, out);
        span_rect(q, -hi, -lo, -ay, ay, out);
    }
    // straight bit on the y axis: from (bx,bb) across to (0,bb)
    let (bx, by) = a[n - 2];
    if by == bb && bx > 0 {
        let (lo, hi) = if inverse { (bb, bb + 2) } else { (a[n - 3].1, bb) };
        span_rect(q, -bx, bx, lo, hi, out);
        span_rect(q, -bx, bx, -hi, -lo, out);
    }
}

/// The grid point strictly inside a chord's box closest to the oval.
fn split_chord(pts: &mut Vec<(i32, i32)>, k: usize, a: f64, b: f64) -> bool {
    let ((x0, y0), (x1, y1)) = (pts[k], pts[k + 1]);
    let mut best: Option<(f64, (i32, i32))> = None;
    let mut x = x1 + 2;
    while x < x0 {
        let mut y = y0 + 2;
        while y < y1 {
            let d = ((x as f64 / a).hypot(y as f64 / b) - 1.0).abs();
            if best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, (x, y)));
            }
            y += 2;
        }
        x += 2;
    }
    match best {
        Some((_, p)) => {
            pts.insert(k + 1, p);
            true
        }
        None => false,
    }
}

/// Doughnut in quarter 1: outer and inner outlines (refined so their wedges
/// fit inside the band), then the filler between them.
fn ring_q1(l: &Layout, fill: Fill) -> Result<Vec<Piece>, String> {
    let mut o = arc(l.width, l.length);
    let mut i = arc(l.hole_w, l.hole_l);
    let (ow, ol, iw, il) = (l.width as f64, l.length as f64, l.hole_w as f64, l.hole_l as f64);
    let _ = (ow, ol);
    // column starts (every grid column in the band, plus the centre line)
    let px = l.width.rem_euclid(2);
    let mut cols: Vec<i32> = vec![0];
    let mut x = px;
    while x < l.width {
        if x > 0 {
            cols.push(x);
        }
        x += 2;
    }
    cols.push(l.width);
    cols.sort();
    cols.dedup();
    // staircase heights for a column starting at x: outer (bottom of the outer
    // wedges) and inner (top of the inner wedges)
    let h_outer = |o: &[(i32, i32)], x: i32| -> (i32, usize) {
        for (k, ((x0, y0), (x1, _))) in segs(o).into_iter().enumerate() {
            if x >= x1 && x < x0 {
                return (y0, k);
            }
        }
        (0, usize::MAX)
    };
    let h_inner = |i: &[(i32, i32)], x: i32| -> (i32, usize) {
        for (k, ((x0, _), (x1, y1))) in segs(i).into_iter().enumerate() {
            if x >= x1 && x < x0 {
                return (y1, k);
            }
        }
        (0, usize::MAX)
    };
    let mut guard = 0;
    loop {
        guard += 1;
        if guard > 100_000 {
            return Err("The band is too thin to build.".into());
        }
        let mut bad = None;
        for w in cols.windows(2) {
            let (ho, ko) = h_outer(&o, w[0]);
            let (hi, ki) = h_inner(&i, w[0]);
            if hi > ho {
                bad = Some((ko, ki));
                break;
            }
        }
        let Some((ko, ki)) = bad else { break };
        // split the bigger of the two chords first, then the other
        let size = |p: &[(i32, i32)], k: usize| if k == usize::MAX { -1 } else { (p[k].0 - p[k + 1].0) + (p[k + 1].1 - p[k].1) };
        let (so, si) = (size(&o, ko), size(&i, ki));
        let mut done = false;
        let order: [bool; 2] = if so >= si { [true, false] } else { [false, true] };
        for outer in order {
            if outer && ko != usize::MAX && split_chord(&mut o, ko, ow, ol) {
                done = true;
                break;
            }
            if !outer && ki != usize::MAX && split_chord(&mut i, ki, iw, il) {
                done = true;
                break;
            }
        }
        if !done {
            return Err("The band is too thin here: the inner and outer edges would overlap. Make the band thicker.".into());
        }
    }
    let mut q1 = outer_wedges(&o);
    q1.extend(inner_wedges(&i));
    // where the four-way style switches from upright to flat strips: columns
    // right of the outer edge's diagonal corner run the full band; left of it
    // they stop at the corner's height and flat strips take over above
    let ((kx, ky), _) = diagonal(&o);
    let diag_t = |x: i32| -> i32 {
        if fill == Fill::Strips || x >= kx { i32::MAX } else { ky }
    };
    // upright strips (columns), merged where identical
    let mut col_runs: Vec<(i32, i32, i32, i32)> = vec![];
    let mut tops: Vec<(i32, i32, i32, i32)> = vec![]; // (x0, x1, lo, hi) above the switch line
    for w in cols.windows(2) {
        let (x0, x1) = (w[0], w[1]);
        let (ho, _) = h_outer(&o, x0);
        let hi = if x0 < l.hole_w { h_inner(&i, x0).0 } else { 0 };
        let t = diag_t(x0);
        let up = ho.min(t.max(hi));
        if up > hi {
            match col_runs.last_mut() {
                Some(r) if r.1 == x0 && r.2 == hi && r.3 == up => r.1 = x1,
                _ => col_runs.push((x0, x1, hi, up)),
            }
        }
        if ho > up.max(hi) {
            tops.push((x0, x1, up.max(hi), ho));
        }
    }
    for (x0, x1, lo, hi) in col_runs {
        q1.push(rect(x0, lo, x1, hi));
    }
    // flat strips (rows) for the part above the switch line
    if !tops.is_empty() {
        let mut ys: Vec<i32> = tops.iter().flat_map(|t| [t.2, t.3]).collect();
        ys.sort();
        ys.dedup();
        let mut row_runs: Vec<(i32, i32, i32, i32)> = vec![]; // (y0, y1, x0, x1)
        for w in ys.windows(2) {
            let (y0, y1) = (w[0], w[1]);
            let xs: Vec<&(i32, i32, i32, i32)> = tops.iter().filter(|t| t.2 <= y0 && t.3 >= y1).collect();
            if xs.is_empty() {
                continue;
            }
            let xa = xs.iter().map(|t| t.0).min().unwrap();
            let xb = xs.iter().map(|t| t.1).max().unwrap();
            match row_runs.last_mut() {
                Some(r) if r.1 == y0 && r.2 == xa && r.3 == xb => r.1 = y1,
                _ => row_runs.push((y0, y1, xa, xb)),
            }
        }
        for (y0, y1, x0, x1) in row_runs {
            q1.push(rect(x0, y0, x1, y1));
        }
    }
    Ok(q1)
}

pub fn layout(l: &Layout) -> Result<Vec<Piece>, String> {
    let a = arc(l.width, l.length);
    let q = quads(l.shape);
    let mut out = vec![];
    match (l.kind, l.inverse) {
        (Kind::Solid, false) => solid(&a, &q, l.fill, &mut out),
        (Kind::Solid, true) => solid_inverse(&a, &q, l.fill, l.wall, &mut out),
        (Kind::Hollow, false) => {
            out.extend(mirror(&outer_wedges(&a), &q));
            axis_bars(&a, &q, false, &mut out);
        }
        (Kind::Hollow, true) => {
            out.extend(mirror(&inner_wedges(&a), &q));
            axis_bars(&a, &q, true, &mut out);
        }
        (Kind::Ring, false) => {
            let q1 = ring_q1(l, l.fill)?;
            out.extend(mirror(&q1, &q));
        }
        (Kind::Ring, true) => {
            // block with a hole the size of the outer edge, plus a disc the size of the hole
            solid_inverse(&a, &q, l.fill, l.wall, &mut out);
            let inner = arc(l.hole_w, l.hole_l);
            solid(&inner, &q, l.fill, &mut out);
        }
    }
    Ok(merge(out))
}

/// Join filler pieces that sit edge to edge with exactly the same span:
/// the two halves of a strip either side of a centre line, or a half-cell
/// sliver that is not a real brick on its own.
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
                    let thin = |w: i32| w % 2 != 0;
                    if a1 == c0 && b0 == e0 && b1 == e1 && (thin(a1 - a0) || thin(c1 - c0) || (a1 == 0 && a0 == -c1)) {
                        ps[i] = rect(a0, b0, c1, b1);
                        ps.remove(j);
                        done = false;
                        break 'outer;
                    }
                    if b1 == e0 && a0 == c0 && a1 == c1 && (thin(b1 - b0) || thin(e1 - e0) || (b1 == 0 && b0 == -e1)) {
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
