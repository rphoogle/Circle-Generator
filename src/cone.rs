//! Cone engine: sloped circles and ovals built from Micro Wedge Triangle Corners,
//! Outer Corners and side-laid Micro Wedges (no Inner Corners - they look bumpy).
//!
//! Plain-English summary of how it works:
//! * Every shape is worked out for one quarter (+x, +y) and then mirrored.
//! * Coordinates are in "half-cells": 1 micro brick = 2 half-cells = 2 game units.
//! * A level is one layer of bricks. Its bottom edge is a staircase outline; each
//!   diagonal step gets a Triangle Corner, an Outer Corner fills the gap between two
//!   Triangle Corners, and side-laid wedges cover the four flat compass sides. The top
//!   edge of a level is the line through the Triangle Corner peaks, which becomes the
//!   bottom edge of the next level up ("exact" stacking, picked by the user as C02a).
//! * Holes (inverse) use the same pieces turned to face the middle.
//! * "Large blocks" fill keeps hidden insides hollow but closed: a floor of big flat
//!   bricks at the bottom, a flat lid on top (with one big flat wedge across the
//!   diagonal corner of holes), and big full-height blocks where possible.

use crate::geometry::{arc, Shape};

pub type Pt = (i32, i32);
/// x0, y0, x1, y1 in half-cells (quarter coordinates)
pub type Bx = (i32, i32, i32, i32);

pub const A_TRI: &str = "PB_DefaultMicroWedgeTriangleCorner";
pub const A_OUT: &str = "PB_DefaultMicroWedgeOuterCorner";
pub const A_WEDGE: &str = "PB_DefaultMicroWedge";
pub const A_BRICK: &str = "PB_DefaultMicroBrick";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    XP,
    XN,
    YP,
    YN,
    ZP,
    ZN,
}

/// One finished brick, game units (half-cells), centre position and half-sizes.
#[derive(Clone, Debug, PartialEq)]
pub struct Cb {
    pub asset: &'static str,
    pub size: (i32, i32, i32),
    pub pos: (i32, i32, i32),
    pub dir: Dir,
    pub rot: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConeType {
    Solid,
    Hollow,
    Inverse,
    Doughnut,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Levels {
    Single,
    Multiple,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConeFill {
    LargeBlocks,
    Strips,
}

/// All sizes in micro bricks (M).
#[derive(Clone, Debug)]
pub struct ConeSettings {
    pub shape: Shape,
    pub ctype: ConeType,
    pub levels: Levels,
    /// bottom edge size (for holes: the narrow bottom opening)
    pub bottom_w: i32,
    pub bottom_l: i32,
    /// top edge size (for holes: the wide opening; for doughnuts: outside of the flat band)
    pub top_w: i32,
    pub top_l: i32,
    pub height: i32,
    /// holes: size of the square block around the hole
    pub block: i32,
    /// doughnuts: width of the flat band
    pub band: i32,
    pub fill: ConeFill,
    pub upside_down: bool,
}

pub struct ConeResult {
    pub bricks: Vec<Cb>,
    pub warnings: Vec<String>,
    pub levels: usize,
    pub step: i32,
    /// actual top size reached (M)
    pub top_actual: (i32, i32),
}

// ------------------------------------------------------------------ outlines

fn cross(o: Pt, a: Pt, b: Pt) -> i64 {
    (a.0 - o.0) as i64 * (b.1 - o.1) as i64 - (a.1 - o.1) as i64 * (b.0 - o.0) as i64
}
fn dedupe(v: Vec<Pt>) -> Vec<Pt> {
    let mut out: Vec<Pt> = vec![];
    for p in v {
        if out.last() != Some(&p) {
            out.push(p);
        }
    }
    out
}
fn strip_collinear(p: &[Pt]) -> Vec<Pt> {
    if p.len() < 3 {
        return p.to_vec();
    }
    let mut out = vec![p[0]];
    for i in 1..p.len() - 1 {
        if cross(*out.last().unwrap(), p[i], p[i + 1]) != 0 {
            out.push(p[i]);
        }
    }
    out.push(*p.last().unwrap());
    out
}
fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}
/// split diagonal edges into their smallest lattice steps
fn subdivide(p: &[Pt]) -> Vec<Pt> {
    let mut out = vec![p[0]];
    for i in 0..p.len() - 1 {
        let (a, b) = (p[i], p[i + 1]);
        let (dx, dy) = ((b.0 - a.0) / 2, (b.1 - a.1) / 2);
        if dx != 0 && dy != 0 {
            let g = gcd(dx, dy);
            for k in 1..g {
                out.push((a.0 + 2 * dx / g * k, a.1 + 2 * dy / g * k));
            }
        }
        out.push(b);
    }
    out
}
fn prim(p: &[Pt]) -> Vec<Pt> {
    subdivide(&strip_collinear(p))
}

/// Outline of an ellipse (half-axes a, b in half-cells, both even) as a staircase where
/// every step has one leg of exactly 1 micro brick - the fine stepping of the user's samples.
pub fn unitleg(a: i32, b: i32) -> Vec<Pt> {
    let tol = 0.3;
    let (ea, eb) = (a as f64 + 2.0 * tol, b as f64 + 2.0 * tol);
    let yof = |x: i32| -> i32 {
        let t = 1.0 - (x as f64 / ea).powi(2);
        if t <= 0.0 { 0 } else { ((eb * t.sqrt() / 2.0).floor() as i32) * 2 }
    };
    let xof = |y: i32| -> i32 {
        let t = 1.0 - (y as f64 / eb).powi(2);
        if t <= 0.0 { 0 } else { ((ea * t.sqrt() / 2.0).floor() as i32) * 2 }
    };
    let mut pts: Vec<Pt> = vec![(a, 0), (0, b)];
    let mut x = 0;
    while x <= a {
        pts.push((x, yof(x).min(b)));
        x += 2;
    }
    let mut y = 0;
    while y <= b {
        pts.push((xof(y).min(a), y));
        y += 2;
    }
    pts.sort();
    pts.dedup();
    let mut front: Vec<Pt> = pts
        .iter()
        .copied()
        .filter(|p| !pts.iter().any(|q| q != p && q.0 >= p.0 && q.1 >= p.1))
        .collect();
    front.sort_by_key(|p| (-p.0, p.1));
    let mut v = vec![(a, 0)];
    v.extend(front);
    v.push((0, b));
    dedupe(v)
}

/// Outline for a single level whose ramps reach `k` micro brick inwards (k > 1).
fn kleg(a: i32, b: i32, k: i32) -> Option<Vec<Pt>> {
    let s = 2 * k;
    let tol = 0.3;
    let (ea, eb) = (a as f64 + 2.0 * tol, b as f64 + 2.0 * tol);
    let yof = |x: i32| -> i32 {
        let t = 1.0 - (x as f64 / ea).powi(2);
        if t <= 0.0 { 0 } else { (((eb * t.sqrt() / 2.0).floor() as i32) * 2).min(b) }
    };
    let xof = |y: i32| -> i32 {
        let t = 1.0 - (y as f64 / eb).powi(2);
        if t <= 0.0 { 0 } else { (((ea * t.sqrt() / 2.0).floor() as i32) * 2).min(a) }
    };
    let cols: Vec<Pt> = (0..=a / s).map(|i| (a - i * s, yof(a - i * s))).collect();
    let rows: Vec<Pt> = (0..=b / s).map(|j| (xof(b - j * s), b - j * s)).collect();
    let mut best: Option<((i32, f64), Vec<Pt>)> = None;
    for i in 1..=cols.len() {
        let st = &cols[..i];
        if st.windows(2).any(|w| w[1].1 < w[0].1) {
            break;
        }
        for j in 1..=rows.len() {
            let mut fl: Vec<Pt> = rows[..j].to_vec();
            fl.reverse();
            if fl.windows(2).any(|w| w[1].0 > w[0].0) {
                continue;
            }
            let (p, q) = (st[i - 1], fl[0]);
            if !(q.0 < p.0 && q.1 > p.1) {
                continue;
            }
            let (dx, dy) = (p.0 - q.0, q.1 - p.1);
            let bad = (dx - s).max(0) + (dy - s).max(0);
            let mut all = st.to_vec();
            all.extend(fl.iter().copied());
            let err: f64 = all
                .iter()
                .map(|&(x, y)| ((x as f64 / a as f64).hypot(y as f64 / b as f64) - 1.0).abs())
                .sum();
            let key = (bad, err);
            if best.as_ref().map_or(true, |(k, _)| key.0 < k.0 || (key.0 == k.0 && key.1 < k.1)) {
                best = Some((key, all));
            }
        }
    }
    let (_, pts) = best?;
    let mut v = vec![(a, 0)];
    v.extend(pts);
    v.push((0, b));
    Some(strip_collinear(&dedupe(v)))
}

/// x of the outline at height y (linear between corners)
pub fn xlim(p: &[Pt], y: f64) -> f64 {
    for i in 0..p.len() - 1 {
        let ((x0, y0), (x1, y1)) = (p[i], p[i + 1]);
        let (y0f, y1f) = (y0 as f64, y1 as f64);
        if y0f <= y && y <= y1f {
            if y1 == y0 {
                return x0.max(x1) as f64;
            }
            return x0 as f64 + (x1 - x0) as f64 * (y - y0f) / (y1f - y0f);
        }
    }
    0.0
}

#[derive(Clone, Debug)]
pub struct Tier {
    /// Triangle Corner boxes (the outline's diagonal steps)
    pub tris: Vec<Bx>,
    /// Outer Corner boxes (between two Triangle Corner peaks)
    pub outers: Vec<Bx>,
    /// side wedge at the +x compass point: (x0, x1, half length y)
    pub wx: Option<(i32, i32, i32)>,
    /// side wedge at the +y compass point: (y0, y1, half length x)
    pub wy: Option<(i32, i32, i32)>,
    pub top: Vec<Pt>,
}

/// The one-level rule: bottom outline -> pieces + top outline.
pub fn tier(b: &[Pt]) -> Result<Tier, String> {
    let mut tris = vec![];
    let mut apex: Vec<Pt> = vec![];
    for i in 0..b.len() - 1 {
        let (p, q) = (b[i], b[i + 1]);
        if q.0 < p.0 && q.1 > p.1 {
            tris.push((q.0, p.1, p.0, q.1));
            apex.push((q.0, p.1));
        }
    }
    if apex.is_empty() {
        return Err("This shape is too small to make a slope - try a bigger size.".into());
    }
    let outers = apex.windows(2).map(|w| (w[1].0, w[0].1, w[0].0, w[1].1)).collect();
    let xm = b[0].0;
    let ym = b[b.len() - 1].1;
    let wx = if b.len() > 1 && b[1].0 == xm && b[1].1 > 0 { Some((apex[0].0, xm, b[1].1)) } else { None };
    let n = b.len();
    let wy = if n > 1 && b[n - 2].1 == ym && b[n - 2].0 > 0 { Some((apex[apex.len() - 1].1, ym, b[n - 2].0)) } else { None };
    let mut top = vec![(apex[0].0, 0)];
    top.extend(apex.iter().copied());
    top.push((0, apex[apex.len() - 1].1));
    Ok(Tier { tris, outers, wx, wy, top: dedupe(top) })
}

// ------------------------------------------------------------------ pieces -> bricks

/// What a piece is, in quarter coordinates; turned into bricks by `emit`.
#[derive(Clone, Debug)]
enum Pc {
    /// corner piece, box, extra rotation (0 = cone, 2 = turned to face the middle)
    Corner(&'static str, Bx, u8),
    /// flat micro wedge (a triangle seen from above), box, right-angle corner in quarter 0
    Flat(Bx, u8),
    /// side wedge at the x compass point: x0, x1, half length, reversed (hole)
    Wx(i32, i32, i32, bool),
    Wy(i32, i32, i32, bool),
    /// plain micro brick
    Rect(Bx),
}

struct Item {
    pc: Pc,
    z0: i32,
    h: i32,
}

const ROTS: [u8; 4] = [0, 1, 2, 3];

fn mirror(b: Bx, q: u8) -> Bx {
    let (mut x0, mut y0, mut x1, mut y1) = b;
    if q == 1 || q == 2 {
        let t = x0;
        x0 = -x1;
        x1 = -t;
    }
    if q == 2 || q == 3 {
        let t = y0;
        y0 = -y1;
        y1 = -t;
    }
    (x0, y0, x1, y1)
}

fn emit(items: &[Item], quads: &[u8]) -> Vec<Cb> {
    let full_x = quads.contains(&0) && quads.contains(&1) || quads.contains(&2) && quads.contains(&3);
    let mut out = vec![];
    for it in items {
        let (z0, h) = (it.z0, it.h);
        match &it.pc {
            Pc::Corner(asset, bx, extra) => {
                for &q in quads {
                    let (x0, y0, x1, y1) = mirror(*bx, q);
                    let (hx, hy) = ((x1 - x0) / 2, (y1 - y0) / 2);
                    let r = (q + extra) % 4;
                    let (sx, sy) = if r % 2 == 1 { (hy, hx) } else { (hx, hy) };
                    out.push(Cb { asset, size: (sx, sy, h), pos: ((x0 + x1) / 2, (y0 + y1) / 2, z0 + h), dir: Dir::ZP, rot: r });
                }
            }
            Pc::Flat(bx, corner) => {
                for &q in quads {
                    let (x0, y0, x1, y1) = mirror(*bx, q);
                    let (hx, hy) = ((x1 - x0) / 2, (y1 - y0) / 2);
                    let r = mirror_corner(*corner, q);
                    let (sx, sy) = if r % 2 == 1 { (hy, hx) } else { (hx, hy) };
                    out.push(Cb { asset: A_WEDGE, size: (sx, sy, h), pos: ((x0 + x1) / 2, (y0 + y1) / 2, z0 + h), dir: Dir::ZP, rot: r });
                }
            }
            Pc::Rect(bx) => {
                for &q in quads {
                    let (x0, y0, x1, y1) = mirror(*bx, q);
                    out.push(Cb { asset: A_BRICK, size: ((x1 - x0) / 2, (y1 - y0) / 2, h), pos: ((x0 + x1) / 2, (y0 + y1) / 2, z0 + h), dir: Dir::ZP, rot: 0 });
                }
            }
            Pc::Wx(x0, x1, ylen, rev) => {
                // +x side wedge spans quarters 0 and 3, -x side spans 1 and 2
                for (side, qa, qb) in [(1i32, 0u8, 3u8), (-1, 1, 2)] {
                    let has = (quads.contains(&qa), quads.contains(&qb));
                    if !has.0 && !has.1 {
                        continue;
                    }
                    let dir = match (side > 0, rev) {
                        (true, false) | (false, true) => Dir::YP,
                        _ => Dir::YN,
                    };
                    let cx = (x0 + x1) / 2 * side;
                    let (len, cy) = match has {
                        (true, true) => (*ylen, 0),
                        (true, false) => (ylen / 2, ylen / 2),
                        _ => (ylen / 2, -ylen / 2),
                    };
                    out.push(Cb { asset: A_WEDGE, size: (h, (x1 - x0) / 2, len), pos: (cx, cy, z0 + h), dir, rot: 0 });
                }
            }
            Pc::Wy(y0, y1, xlen, rev) => {
                for (side, qa, qb) in [(1i32, 0u8, 1u8), (-1, 3, 2)] {
                    let has = (quads.contains(&qa), quads.contains(&qb));
                    if !has.0 && !has.1 {
                        continue;
                    }
                    let dir = match (side > 0, rev) {
                        (true, false) | (false, true) => Dir::XN,
                        _ => Dir::XP,
                    };
                    let cy = (y0 + y1) / 2 * side;
                    let (len, cx) = match has {
                        (true, true) => (*xlen, 0),
                        (true, false) => (xlen / 2, xlen / 2),
                        _ => (xlen / 2, -xlen / 2),
                    };
                    out.push(Cb { asset: A_WEDGE, size: (h, (y1 - y0) / 2, len), pos: (cx, cy, z0 + h), dir, rot: 0 });
                }
            }
        }
    }
    let _ = full_x;
    merge_rects(out)
}

/// corner index after mirroring a quarter-0 piece into quarter q
fn mirror_corner(c: u8, q: u8) -> u8 {
    // corners: 0 (-x,-y), 1 (+x,-y), 2 (+x,+y), 3 (-x,+y)
    let (mut sx, mut sy) = match c { 0 => (-1, -1), 1 => (1, -1), 2 => (1, 1), _ => (-1, 1) };
    if q == 1 || q == 2 {
        sx = -sx;
    }
    if q == 2 || q == 3 {
        sy = -sy;
    }
    match (sx, sy) { (-1, -1) => 0, (1, -1) => 1, (1, 1) => 2, _ => 3 }
}

/// Merge plain bricks that share a whole edge (same height) into one bigger brick.
fn merge_rects(mut v: Vec<Cb>) -> Vec<Cb> {
    loop {
        let mut merged = false;
        'outer: for i in 0..v.len() {
            if v[i].asset != A_BRICK {
                continue;
            }
            for j in i + 1..v.len() {
                if v[j].asset != A_BRICK {
                    continue;
                }
                let (a, b) = (&v[i], &v[j]);
                if a.pos.2 != b.pos.2 || a.size.2 != b.size.2 {
                    continue;
                }
                let (ax0, ax1, ay0, ay1) = (a.pos.0 - a.size.0, a.pos.0 + a.size.0, a.pos.1 - a.size.1, a.pos.1 + a.size.1);
                let (bx0, bx1, by0, by1) = (b.pos.0 - b.size.0, b.pos.0 + b.size.0, b.pos.1 - b.size.1, b.pos.1 + b.size.1);
                let nb = if ay0 == by0 && ay1 == by1 && (ax1 == bx0 || bx1 == ax0) {
                    Some((ax0.min(bx0), ax1.max(bx1), ay0, ay1))
                } else if ax0 == bx0 && ax1 == bx1 && (ay1 == by0 || by1 == ay0) {
                    Some((ax0, ax1, ay0.min(by0), ay1.max(by1)))
                } else {
                    None
                };
                if let Some((x0, x1, y0, y1)) = nb {
                    if (x1 - x0) / 2 > crate::MAX_HALF_PUB || (y1 - y0) / 2 > crate::MAX_HALF_PUB {
                        continue;
                    }
                    let z = a.pos.2;
                    let h = a.size.2;
                    v[i] = Cb { asset: A_BRICK, size: ((x1 - x0) / 2, (y1 - y0) / 2, h), pos: ((x0 + x1) / 2, (y0 + y1) / 2, z), dir: Dir::ZP, rot: 0 };
                    v.swap_remove(j);
                    merged = true;
                    break 'outer;
                }
            }
        }
        if !merged {
            return v;
        }
    }
}

// ------------------------------------------------------------------ helpers for fill

/// rows of a quarter region: for each 2-half-cell row from y=0, the x range [x0, x1)
fn rows_to_rects(rows: &[(i32, i32, i32)]) -> Vec<Bx> {
    // rows: (y0, xa, xb) with y1 = y0 + 2; merge vertically equal ranges
    let mut out: Vec<Bx> = vec![];
    for &(y0, xa, xb) in rows {
        if xb <= xa {
            continue;
        }
        if let Some(last) = out.iter_mut().rev().find(|r| r.0 == xa && r.2 == xb && r.3 == y0) {
            last.3 = y0 + 2;
            continue;
        }
        out.push((xa, y0, xb, y0 + 2));
    }
    out
}

/// "Biggest bricks" cover of a quarter staircase region anchored at the centre:
/// ext[j] = how far row j (y from 2j to 2j+2) reaches in x. Returns quarter boxes.
fn staircase_cover(ext: &[i32]) -> Vec<Bx> {
    if ext.is_empty() {
        return vec![];
    }
    // best central rectangle
    let mut best = (0i64, 0usize, 0i32);
    let mut m = i32::MAX;
    for j in 0..ext.len() {
        m = m.min(ext[j]);
        let area = m as i64 * (j as i64 + 1) * 2;
        if area > best.0 {
            best = (area, j + 1, m);
        }
    }
    let (_, rows_c, cx) = best;
    let mut out = vec![];
    if cx > 0 {
        out.push((0, 0, cx, 2 * rows_c as i32));
    }
    // beside the centre rectangle
    let side: Vec<(i32, i32, i32)> = (0..rows_c).map(|j| (2 * j as i32, cx, ext[j])).collect();
    out.extend(rows_to_rects(&side));
    // above it
    let above: Vec<(i32, i32, i32)> = (rows_c..ext.len()).map(|j| (2 * j as i32, 0, ext[j])).collect();
    out.extend(rows_to_rects(&above));
    out
}

/// edge of a set of piece boxes per row: min x0 (inner edge) or max x1 (outer edge)
fn row_edge(boxes: &[Bx], y0: i32, inner: bool) -> Option<i32> {
    let it = boxes.iter().filter(|b| b.1 < y0 + 2 && b.3 > y0);
    if inner { it.map(|b| b.0).min() } else { it.map(|b| b.2).max() }
}

fn tier_boxes(t: &Tier) -> Vec<Bx> {
    let mut v: Vec<Bx> = t.tris.iter().chain(t.outers.iter()).copied().collect();
    if let Some((x0, x1, yl)) = t.wx {
        v.push((x0, 0, x1, yl));
    }
    if let Some((y0, y1, xl)) = t.wy {
        v.push((0, y0, xl, y1));
    }
    v
}

fn push_tier(items: &mut Vec<Item>, t: &Tier, z0: i32, h: i32, hole: bool) {
    let extra = if hole { 2 } else { 0 };
    for &b in &t.tris {
        // cones: Triangle Corners on the outline; holes: Outer Corners there
        items.push(Item { pc: Pc::Corner(if hole { A_OUT } else { A_TRI }, b, extra), z0, h });
    }
    for &b in &t.outers {
        items.push(Item { pc: Pc::Corner(if hole { A_TRI } else { A_OUT }, b, extra), z0, h });
    }
    if let Some((x0, x1, yl)) = t.wx {
        items.push(Item { pc: Pc::Wx(x0, x1, yl, hole), z0, h });
    }
    if let Some((y0, y1, xl)) = t.wy {
        items.push(Item { pc: Pc::Wy(y0, y1, xl, hole), z0, h });
    }
}

fn split_heights(total: i32, n: usize) -> Vec<i32> {
    let n32 = n as i32;
    (0..n32).map(|i| total / n32 + if i < total % n32 { 1 } else { 0 }).collect()
}

fn quads_for(shape: Shape) -> Vec<u8> {
    match shape {
        Shape::Full => ROTS.to_vec(),
        Shape::Half => vec![0, 1],
        Shape::Quarter => vec![0],
    }
}

// ------------------------------------------------------------------ main entry

/// Build the stacked levels from a bottom outline. Returns tiers and the scale used.
fn stack(b0: Vec<Pt>, n: usize) -> Result<Vec<Tier>, String> {
    let mut b = prim(&b0);
    let mut out = vec![];
    for _ in 0..n {
        let t = tier(&b)?;
        b = prim(&t.top);
        out.push(t);
    }
    Ok(out)
}

fn scale_tier(t: &Tier, s: i32) -> Tier {
    let sb = |b: &Bx| (b.0 * s, b.1 * s, b.2 * s, b.3 * s);
    Tier {
        tris: t.tris.iter().map(sb).collect(),
        outers: t.outers.iter().map(sb).collect(),
        wx: t.wx.map(|(a, b, c)| (a * s, b * s, c * s)),
        wy: t.wy.map(|(a, b, c)| (a * s, b * s, c * s)),
        top: t.top.iter().map(|p| (p.0 * s, p.1 * s)).collect(),
    }
}

pub fn validate(c: &ConeSettings) -> Result<(), String> {
    if c.height < 1 {
        return Err("The height must be at least 1 micro brick.".into());
    }
    if c.height > crate::MAX_HALF_PUB {
        return Err("That is taller than the game allows (4000 micro bricks).".into());
    }
    for v in [c.bottom_w, c.bottom_l, c.top_w, c.top_l] {
        if v < 4 {
            return Err("Sizes must be at least 4 micro bricks.".into());
        }
    }
    let (bw, bl, tw, tl) = (c.bottom_w, c.bottom_l, c.top_w, c.top_l);
    let (wide_w, wide_l, nar_w, nar_l) = if c.ctype == ConeType::Inverse { (tw, tl, bw, bl) } else { (bw, bl, tw, tl) };
    if c.ctype == ConeType::Inverse {
        if tw <= bw || tl <= bl {
            return Err("For a hole, the top opening must be wider than the bottom.".into());
        }
    } else if bw <= tw || bl <= tl {
        return Err("The bottom must be wider than the top (tick Upside down to flip it).".into());
    }
    if wide_w - nar_w != wide_l - nar_l {
        return Err(format!(
            "The slope must step in by the same amount on both sides: the width shrinks by {} but the length by {}. \
             Make both differences the same.",
            wide_w - nar_w,
            wide_l - nar_l
        ));
    }
    if (wide_w - nar_w) % 2 != 0 {
        return Err("Top and bottom must differ by an even number (the slope steps in on both sides).".into());
    }
    match c.shape {
        Shape::Full => {}
        _ => {
            for v in [bw, bl, tw, tl] {
                if v % 2 != 0 {
                    return Err("Half and quarter shapes need even sizes, because the cut runs through the centre.".into());
                }
            }
        }
    }
    if c.ctype == ConeType::Inverse {
        if c.block < tw + 2 || c.block < tl + 2 {
            return Err("The block must be at least 2 micro bricks bigger than the top opening.".into());
        }
        if c.block % 2 != tw % 2 || c.block % 2 != tl % 2 {
            return Err("The block and the top opening must both be odd or both be even (so the wall is the same on each side).".into());
        }
        if c.shape != Shape::Full && c.block % 2 != 0 {
            return Err("Half and quarter shapes need an even block size.".into());
        }
    }
    if c.ctype == ConeType::Doughnut {
        if c.band < 1 {
            return Err("The band must be at least 1 micro brick wide.".into());
        }
        if tw - 2 * c.band < 2 || tl - 2 * c.band < 2 {
            return Err("The band is too wide - it would close up the hole.".into());
        }
    }
    let _ = (nar_w, nar_l);
    Ok(())
}

/// Works out the cone in even sizes, then widens by one micro brick through the middle
/// for odd sizes (the same trick agreed for the 113 M doughnut).
pub fn build_cone(c: &ConeSettings) -> Result<ConeResult, String> {
    validate(c)?;
    let odd_x = c.bottom_w % 2 != 0;
    let odd_y = c.bottom_l % 2 != 0;
    let mut e = c.clone();
    if odd_x {
        e.bottom_w -= 1;
        e.top_w -= 1;
        if e.ctype == ConeType::Inverse {
            e.block -= 1;
        }
    }
    if odd_y {
        e.bottom_l -= 1;
        e.top_l -= 1;
        if e.ctype == ConeType::Inverse {
            e.block -= 1;
        }
    }
    let mut r = build_even(&e)?;
    if odd_y {
        r.bricks = stretch(r.bricks, 1)?;
        r.top_actual.1 += 1;
    }
    if odd_x {
        r.bricks = stretch(r.bricks, 0)?;
        r.top_actual.0 += 1;
    }
    if c.upside_down {
        r.bricks = flip(r.bricks, c.height);
    }
    for b in &r.bricks {
        if b.size.0 > crate::MAX_HALF_PUB || b.size.1 > crate::MAX_HALF_PUB || b.size.2 > crate::MAX_HALF_PUB {
            return Err("One of the bricks would be bigger than the game allows - try a smaller size.".into());
        }
        if b.size.0 < 1 || b.size.1 < 1 || b.size.2 < 1 {
            return Err(format!("internal: zero-sized brick {b:?}"));
        }
    }
    Ok(r)
}

fn build_even(c: &ConeSettings) -> Result<ConeResult, String> {
    let quads = quads_for(c.shape);
    let mut warnings = vec![];
    let hole = c.ctype == ConeType::Inverse;
    // sizes in half-cells (diameter in M == half-axis in half-cells)
    let (wide_w, wide_l, nar_w) = if hole { (c.top_w, c.top_l, c.bottom_w) } else { (c.bottom_w, c.bottom_l, c.top_w) };
    let steps = (wide_w - nar_w) / 2; // micro bricks per side
    let mut items: Vec<Item> = vec![];
    let tiers: Vec<Tier>;
    let hs: Vec<i32>;
    let mut scale = 1;
    match c.levels {
        Levels::Single => {
            let b = if steps == 1 {
                prim(&unitleg(wide_w, wide_l))
            } else {
                warnings.push(format!(
                    "Single level with {steps} M ramps: each corner brick has to reach {steps} M, so the curve is made of fewer, bigger flat facets. Multiple levels will look rounder."
                ));
                kleg(wide_w, wide_l, steps).ok_or("These sizes can't make a single-level slope - try Multiple levels.")?
            };
            let t = tier(&b)?;
            let (tx, ty) = (t.top[0].0, t.top[t.top.len() - 1].1);
            if tx != wide_w - 2 * steps || ty != wide_l - 2 * steps {
                return Err(format!(
                    "{steps} M ramps are too wide for a single level at this size (the top would come out {tx}×{ty} instead of {}×{}). Try Multiple levels, or a smaller difference between top and bottom.",
                    wide_w - 2 * steps,
                    wide_l - 2 * steps
                ));
            }
            tiers = vec![t];
            hs = vec![c.height];
        }
        Levels::Multiple => {
            if c.height < steps {
                scale = (steps + c.height - 1) / c.height;
                while steps % scale != 0 || wide_w % (scale) != 0 || wide_l % scale != 0 || (wide_w / scale) % 2 != 0 || (wide_l / scale) % 2 != 0 {
                    scale += 1;
                    if scale > steps {
                        return Err("This shape is too short for its width, even with bigger steps. Make it taller.".into());
                    }
                }
                warnings.push(format!(
                    "Chunky: the shape is only {} M tall but steps in {} M, so each level steps in {} M instead of 1 M. Make it at least {} M tall for smooth 1 M steps.",
                    c.height, steps, scale, steps
                ));
            }
            let n = (steps / scale) as usize;
            let t0 = stack(unitleg(wide_w / scale, wide_l / scale), n)?;
            tiers = t0.iter().map(|t| scale_tier(t, scale)).collect();
            hs = split_heights(c.height, n);
        }
    }
    let last = tiers.last().unwrap();
    let top_actual = (last.top[0].0, last.top[last.top.len() - 1].1);
    let n = tiers.len();
    // z: cones build upwards from level 0; holes have the wide level at the top
    let mut zs = vec![0; n];
    {
        let mut z = 0;
        if hole {
            for k in (0..n).rev() {
                zs[k] = z;
                z += 2 * hs[k];
            }
        } else {
            for k in 0..n {
                zs[k] = z;
                z += 2 * hs[k];
            }
        }
    }
    let total = 2 * c.height;
    for k in 0..n {
        push_tier(&mut items, &tiers[k], zs[k], hs[k], hole);
    }
    let large = c.fill == ConeFill::LargeBlocks;
    match c.ctype {
        ConeType::Hollow => {}
        ConeType::Solid => solid_fill(&mut items, &tiers, &zs, &hs, total, large),
        ConeType::Inverse => hole_fill(&mut items, &tiers, &zs, &hs, total, c.block, large),
        ConeType::Doughnut => doughnut_fill(&mut items, &tiers, &zs, &hs, total, c, large, &mut warnings)?,
    }
    let bricks = emit(&items, &quads);
    Ok(ConeResult { bricks, warnings, levels: n, step: scale, top_actual })
}

/// inner edge of each level per row (where the slope bricks start)
fn inner_ext(t: &Tier, rows: usize) -> Vec<i32> {
    let bx = tier_boxes(t);
    (0..rows)
        .map(|j| {
            let y0 = 2 * j as i32;
            match row_edge(&bx, y0, true) {
                Some(e) => e,
                None => {
                    // above all pieces of this level (beyond its top): nothing
                    if y0 < t.top[t.top.len() - 1].1 { xlim(&t.top, y0 as f64 + 1.0) as i32 } else { 0 }
                }
            }
        })
        .collect()
}

fn solid_fill(items: &mut Vec<Item>, tiers: &[Tier], zs: &[i32], hs: &[i32], total: i32, large: bool) {
    let n = tiers.len();
    let rows_of = |t: &Tier| (t.top[t.top.len() - 1].1.max(0) / 2) as usize + 1;
    if !large || total <= 4 {
        // Strips: every level filled solid
        for k in 0..n {
            let ext = inner_ext(&tiers[k], rows_of(&tiers[k]));
            for b in staircase_cover(&ext) {
                items.push(Item { pc: Pc::Rect(b), z0: zs[k], h: hs[k] });
            }
        }
        return;
    }
    // Large blocks: floor (1 M) at the bottom of level 0, lid (1 M) at the top of the last level.
    let ext0 = inner_ext(&tiers[0], rows_of(&tiers[0]));
    for b in staircase_cover(&ext0) {
        items.push(Item { pc: Pc::Rect(b), z0: 0, h: 1 });
    }
    let extn = inner_ext(&tiers[n - 1], rows_of(&tiers[n - 1]));
    for b in staircase_cover(&extn) {
        items.push(Item { pc: Pc::Rect(b), z0: total - 2, h: 1 });
    }
}

/// holes: frame of big full-height blocks outside the opening, hollow corners closed
/// by a lid (big diagonal flat wedge + small wedges + bricks) and a floor.
fn hole_fill(items: &mut Vec<Item>, tiers: &[Tier], zs: &[i32], hs: &[i32], total: i32, block: i32, large: bool) {
    let n = tiers.len();
    let s = block; // half-cells to the block edge
    let top = &tiers[0];
    let (rx, ry) = (top.wx.map(|w| w.1).unwrap_or(top.tris[0].2), top.wy.map(|w| w.1).unwrap_or(top.tris[top.tris.len() - 1].3));
    // outer edge of each level per row
    let outer_ext = |t: &Tier, y0: i32| -> i32 {
        let bx = tier_boxes(t);
        row_edge(&bx, y0, false).unwrap_or(0)
    };
    if !large || total <= 4 {
        for k in 0..n {
            let rows: Vec<(i32, i32, i32)> = (0..s / 2).map(|j| (2 * j, outer_ext(&tiers[k], 2 * j), s)).collect();
            for b in rows_to_rects(&rows) {
                items.push(Item { pc: Pc::Rect(b), z0: zs[k], h: hs[k] });
            }
        }
        return;
    }
    let full = total / 2;
    // frame: everything outside the rx x ry rectangle, full height
    if s > ry {
        items.push(Item { pc: Pc::Rect((0, ry, s, s)), z0: 0, h: full });
    }
    if s > rx {
        items.push(Item { pc: Pc::Rect((rx, 0, s, ry)), z0: 0, h: full });
    }
    // floor: bottom level, from its outer edge to the rectangle
    let bottom = &tiers[n - 1];
    let floor: Vec<(i32, i32, i32)> = (0..ry / 2).map(|j| (2 * j, outer_ext(bottom, 2 * j), rx)).collect();
    for b in rows_to_rects(&floor) {
        items.push(Item { pc: Pc::Rect(b), z0: 0, h: 1 });
    }
    // lid: top level (top 1 M), corner region with a big diagonal wedge
    let start: Vec<i32> = (0..ry / 2).map(|j| outer_ext(top, 2 * j)).collect();
    lid_corner(items, &start, rx, ry, total - 2);
}

/// cover the corner region {row j: x from start[j] to rx} of a quarter (cells of 1 M)
/// with one big flat wedge on the diagonal, small wedges along it, and bricks.
fn lid_corner(items: &mut Vec<Item>, start: &[i32], rx: i32, ry: i32, z0: i32) {
    let (nx, ny) = (rx / 2, ry / 2);
    let inreg = |i: i32, j: i32| -> bool { j >= 0 && j < ny && i < nx && i >= 0 && 2 * i >= start[j as usize] };
    // largest triangle with its right angle at the rectangle corner
    let mut best_l = 0;
    for l in 1..=nx.min(ny) {
        let mut ok = true;
        'c: for j in ny - l..ny {
            for i in nx - l..nx {
                let d = (nx - i) + (ny - j);
                if d <= l + 1 && !inreg(i, j) {
                    ok = false;
                    break 'c;
                }
            }
        }
        if ok {
            best_l = l;
        } else {
            break;
        }
    }
    let mut covered = vec![vec![false; nx.max(0) as usize]; ny.max(0) as usize];
    if best_l >= 2 {
        let l = best_l;
        items.push(Item { pc: Pc::Flat((2 * (nx - l), 2 * (ny - l), 2 * nx, 2 * ny), 2), z0, h: 1 });
        for j in ny - l..ny {
            for i in nx - l..nx {
                let d = (nx - i) + (ny - j);
                if d <= l {
                    covered[j as usize][i as usize] = true;
                } else if d == l + 1 {
                    // the cell is cut by the long edge: fill its other half with a small wedge
                    items.push(Item { pc: Pc::Flat((2 * i, 2 * j, 2 * i + 2, 2 * j + 2), 0), z0, h: 1 });
                    covered[j as usize][i as usize] = true;
                }
            }
        }
    }
    let mut rows = vec![];
    for j in 0..ny {
        let mut i = 0;
        while i < nx {
            if inreg(i, j) && !covered[j as usize][i as usize] {
                let a = i;
                while i < nx && inreg(i, j) && !covered[j as usize][i as usize] {
                    i += 1;
                }
                rows.push((2 * j, 2 * a, 2 * i));
            } else {
                i += 1;
            }
        }
    }
    // merge rows into rectangles (vertical runs with the same range)
    rows.sort_by_key(|r| (r.1, r.2, r.0));
    let mut rects: Vec<Bx> = vec![];
    for (y0, xa, xb) in rows {
        if let Some(r) = rects.iter_mut().find(|r| r.0 == xa && r.2 == xb && r.3 == y0) {
            r.3 = y0 + 2;
        } else {
            rects.push((xa, y0, xb, y0 + 2));
        }
    }
    for b in rects {
        items.push(Item { pc: Pc::Rect(b), z0, h: 1 });
    }
}

/// sloped doughnut: flat band (full height) around a smooth hole, slope round the outside.
fn doughnut_fill(
    items: &mut Vec<Item>,
    tiers: &[Tier],
    zs: &[i32],
    hs: &[i32],
    total: i32,
    c: &ConeSettings,
    large: bool,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let n = tiers.len();
    let (hx, hy) = (c.top_w - 2 * c.band, c.top_l - 2 * c.band);
    let hole = arc(hx, hy);
    let mut hboxes: Vec<Bx> = vec![];
    for i in 0..hole.len() - 1 {
        let ((x0, y0), (x1, y1)) = (hole[i], hole[i + 1]);
        if x1 < x0 && y1 > y0 {
            hboxes.push((x1, y0, x0, y1));
        }
    }
    let full = total / 2;
    for &b in &hboxes {
        // hole edge: flat micro wedge with the material outside (right angle at the outer corner)
        items.push(Item { pc: Pc::Flat(b, 2), z0: 0, h: full });
    }
    let hole_edge = |y0: i32| -> Result<i32, String> {
        if y0 >= hy {
            return Ok(0);
        }
        if let Some(e) = hboxes.iter().filter(|b| b.1 < y0 + 2 && b.3 > y0).map(|b| b.2).max() {
            return Ok(e);
        }
        let v = xlim(&hole, y0 as f64 + 1.0);
        if v.fract() != 0.0 || (v as i32) % 2 != 0 {
            return Err("internal: hole edge off the grid".into());
        }
        Ok(v as i32)
    };
    let topt = &tiers[n - 1];
    let ymax = tier_boxes(&tiers[0]).iter().map(|b| b.3).max().unwrap_or(0);
    let inner = |t: &Tier, y0: i32| -> Option<i32> { row_edge(&tier_boxes(t), y0, true) };
    let mut core_rows = vec![];
    let mut extra: Vec<(usize, i32, i32, i32)> = vec![];
    let mut floor_rows = vec![];
    for y0 in (0..ymax).step_by(2) {
        let he = hole_edge(y0)?;
        let core_out = inner(topt, y0).unwrap_or(0);
        if core_out < he {
            return Err("The slope's corner bricks would cut into the flat band at this size. Make the band wider, or use Multiple levels.".into());
        }
        if core_out > he {
            core_rows.push((y0, he, core_out));
        }
        // between the core and each lower level's slope
        for k in 0..n - 1 {
            if let Some(e) = inner(&tiers[k], y0) {
                if e > core_out {
                    if large {
                        if k == 0 {
                            floor_rows.push((y0, core_out, e));
                        }
                    } else {
                        extra.push((k, y0, core_out, e));
                    }
                }
            }
        }
    }
    for b in rows_to_rects(&core_rows) {
        items.push(Item { pc: Pc::Rect(b), z0: 0, h: full });
    }
    for b in rows_to_rects(&floor_rows) {
        items.push(Item { pc: Pc::Rect(b), z0: 0, h: 1 });
    }
    for k in 0..n {
        let rows: Vec<(i32, i32, i32)> = extra.iter().filter(|e| e.0 == k).map(|e| (e.1, e.2, e.3)).collect();
        for b in rows_to_rects(&rows) {
            items.push(Item { pc: Pc::Rect(b), z0: zs[k], h: hs[k] });
        }
    }
    let t = &tiers[n - 1].top;
    if t[0].0 != c.top_w - (c.top_w % 2) || t[t.len() - 1].1 != c.top_l - (c.top_l % 2) {
        warnings.push(format!("The flat band's outside edge comes out {}×{} M.", t[0].0, t[t.len() - 1].1));
    }
    Ok(())
}

// ------------------------------------------------------------------ odd sizes and flipping

fn footprint(b: &Cb) -> (i32, i32, i32) {
    match b.dir {
        Dir::ZP | Dir::ZN => {
            if b.rot % 2 == 1 { (b.size.1, b.size.0, b.size.2) } else { (b.size.0, b.size.1, b.size.2) }
        }
        Dir::YP | Dir::YN => (b.size.1, b.size.2, b.size.0),
        Dir::XP | Dir::XN => (b.size.2, b.size.1, b.size.0),
    }
}

/// Widen by 1 micro brick through the middle along axis (0 = x, 1 = y).
fn stretch(bricks: Vec<Cb>, axis: usize) -> Result<Vec<Cb>, String> {
    let g = |b: &Cb, a: usize| if a == 0 { b.pos.0 } else { b.pos.1 };
    let half = |b: &Cb, a: usize| { let f = footprint(b); if a == 0 { f.0 } else { f.1 } };
    let mut out = vec![];
    for mut b in bricks {
        let (c, h) = (g(&b, axis), half(&b, axis));
        let (lo, hi) = (c - h, c + h);
        if lo >= 0 {
            if axis == 0 { b.pos.0 += 1 } else { b.pos.1 += 1 }
        } else if hi <= 0 {
            if axis == 0 { b.pos.0 -= 1 } else { b.pos.1 -= 1 }
        } else {
            // crosses the middle: grow by one unit each side
            if c != 0 {
                return Err("This odd size can't be made with this combination - try one size up or down.".into());
            }
            match b.dir {
                Dir::ZP | Dir::ZN => {
                    let swap = b.rot % 2 == 1;
                    if (axis == 0) != swap { b.size.0 += 1 } else { b.size.1 += 1 }
                }
                Dir::YP | Dir::YN => {
                    if axis == 0 { b.size.1 += 1 } else { b.size.2 += 1 }
                }
                Dir::XP | Dir::XN => {
                    if axis == 0 { b.size.2 += 1 } else { b.size.1 += 1 }
                }
            }
        }
        out.push(b);
    }
    // pieces that touch the middle line from one side leave a 1-unit gap; only plain
    // bricks can be widened, and they come in mirrored pairs - merge those first.
    let mut v = out;
    let mut i = 0;
    while i < v.len() {
        let b = v[i].clone();
        let (c, h) = (g(&b, axis), half(&b, axis));
        let touching = (c - h == 1) || (c + h == -1);
        if touching {
            if b.asset != A_BRICK {
                return Err("This odd size can't be made with this combination - try one size up or down.".into());
            }
            // find its mirror partner
            let j = v.iter().position(|o| {
                o.asset == A_BRICK && o.pos.2 == b.pos.2 && o.size == b.size && g(o, axis) == -c && (if axis == 0 { o.pos.1 == b.pos.1 } else { o.pos.0 == b.pos.0 })
            });
            match j {
                Some(j) => {
                    let mut m = b.clone();
                    if axis == 0 {
                        m.pos.0 = 0;
                        m.size.0 = c.abs() + h;
                    } else {
                        m.pos.1 = 0;
                        m.size.1 = c.abs() + h;
                    }
                    let (a, bb) = if i < j { (i, j) } else { (j, i) };
                    v.remove(bb);
                    v.remove(a);
                    v.push(m);
                    i = 0;
                    continue;
                }
                None => return Err("This odd size can't be made with this combination - try one size up or down.".into()),
            }
        }
        i += 1;
    }
    Ok(v)
}

/// Upside down: mirror every brick top-to-bottom. Mapping read from the user's
/// flipped C01 copied out of the game (19 Nov 2026):
/// corner bricks ZPositive DegR (a, b) -> ZNegative Deg[1,0,3,2][R] (b, a);
/// side wedges keep their size, reverse direction and turn 180 degrees;
/// plain bricks and flat wedges look the same either way up.
fn flip(bricks: Vec<Cb>, height: i32) -> Vec<Cb> {
    let total = 2 * height;
    bricks
        .into_iter()
        .map(|mut b| {
            b.pos.2 = total - b.pos.2;
            match b.dir {
                Dir::ZP => {
                    if b.asset == A_TRI || b.asset == A_OUT {
                        b.dir = Dir::ZN;
                        b.rot = [1, 0, 3, 2][b.rot as usize];
                        b.size = (b.size.1, b.size.0, b.size.2);
                    }
                }
                Dir::YP => { b.dir = Dir::YN; b.rot = (b.rot + 2) % 4; }
                Dir::YN => { b.dir = Dir::YP; b.rot = (b.rot + 2) % 4; }
                Dir::XP => { b.dir = Dir::XN; b.rot = (b.rot + 2) % 4; }
                Dir::XN => { b.dir = Dir::XP; b.rot = (b.rot + 2) % 4; }
                Dir::ZN => {}
            }
            b
        })
        .collect()
}

/// Quick self-check: no two bricks' boxes overlap.
pub fn overlaps(bricks: &[Cb]) -> usize {
    use std::collections::HashMap;
    let boxes: Vec<(i32, i32, i32, i32, i32, i32)> = bricks
        .iter()
        .map(|b| {
            let (hx, hy, hz) = footprint(b);
            (b.pos.0 - hx, b.pos.1 - hy, b.pos.2 - hz, b.pos.0 + hx, b.pos.1 + hy, b.pos.2 + hz)
        })
        .collect();
    let g = 32;
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    let mut bad = std::collections::HashSet::new();
    for (i, a) in boxes.iter().enumerate() {
        for gx in a.0.div_euclid(g)..=(a.3 - 1).div_euclid(g) {
            for gy in a.1.div_euclid(g)..=(a.4 - 1).div_euclid(g) {
                let e = grid.entry((gx, gy)).or_default();
                for &j in e.iter() {
                    let c = boxes[j];
                    if a.0 < c.3 && c.0 < a.3 && a.1 < c.4 && c.1 < a.4 && a.2 < c.5 && c.2 < a.5 {
                        if !polys_overlap(&poly(&bricks[i], &boxes[i]), &poly(&bricks[j], &boxes[j])) {
                            continue;
                        }
                        bad.insert((j, i));
                    }
                }
                e.push(i);
            }
        }
    }
    bad.len()
}

/// Footprint seen from above (triangle for flat wedges and Triangle Corners).
fn poly(b: &Cb, bb: &(i32, i32, i32, i32, i32, i32)) -> Vec<(f64, f64)> {
    let (x0, y0, x1, y1) = (bb.0 as f64, bb.1 as f64, bb.3 as f64, bb.4 as f64);
    let c = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)];
    if b.dir == Dir::ZP && (b.asset == A_WEDGE || b.asset == A_TRI) {
        let r = b.rot as usize;
        // right angle at corner r, keep r and its two neighbours
        vec![c[r], c[(r + 1) % 4], c[(r + 3) % 4]]
    } else {
        c.to_vec()
    }
}
/// separating-axis test for two convex polygons (touching edges do not count)
fn polys_overlap(a: &[(f64, f64)], b: &[(f64, f64)]) -> bool {
    for poly in [a, b] {
        for i in 0..poly.len() {
            let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
            let n = (q.1 - p.1, p.0 - q.0);
            let proj = |v: &[(f64, f64)]| {
                let mut lo = f64::MAX;
                let mut hi = f64::MIN;
                for &(x, y) in v {
                    let d = x * n.0 + y * n.1;
                    lo = lo.min(d);
                    hi = hi.max(d);
                }
                (lo, hi)
            };
            let (a0, a1) = proj(a);
            let (b0, b1) = proj(b);
            if a1 <= b0 + 1e-9 || b1 <= a0 + 1e-9 {
                return false;
            }
        }
    }
    true
}

/// Footprint seen from above, for the preview.
pub fn footprint_poly(b: &Cb) -> Vec<(f64, f64)> {
    let (hx, hy, hz) = footprint(b);
    let bb = (b.pos.0 - hx, b.pos.1 - hy, b.pos.2 - hz, b.pos.0 + hx, b.pos.1 + hy, b.pos.2 + hz);
    poly(b, &bb)
}
/// Top of a brick (game units), for the preview's shading.
pub fn top_z(b: &Cb) -> i32 {
    b.pos.2 + footprint(b).2
}
