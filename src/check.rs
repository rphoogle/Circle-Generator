//! Self-check: samples the layout on a fine grid and confirms there are no
//! overlaps, no gaps, and nothing outside where the shape should be.

use crate::geometry::{arc, Kind, Layout, Piece, Shape};

#[derive(Debug, Default)]
pub struct Report {
    pub overlaps: usize,
    pub gaps: usize,
    pub spill: usize,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.overlaps == 0 && self.gaps == 0 && self.spill == 0
    }
}

fn inside_convex(poly: &[(f64, f64)], x: f64, y: f64) -> bool {
    let n = poly.len();
    let mut sign = 0i8;
    for i in 0..n {
        let (x0, y0) = poly[i];
        let (x1, y1) = poly[(i + 1) % n];
        let c = (x1 - x0) * (y - y0) - (y1 - y0) * (x - x0);
        let s = if c > 0.0 { 1 } else if c < 0.0 { -1 } else { 0 };
        if s != 0 {
            if sign == 0 {
                sign = s;
            } else if s != sign {
                return false;
            }
        }
    }
    true
}

pub fn check(l: &Layout, pieces: &[Piece]) -> Report {
    check_with_step(l, pieces, 0.5)
}

pub fn check_with_step(l: &Layout, pieces: &[Piece], step: f64) -> Report {
    let d = l.diameter;
    let a = arc(d);
    // full outline, anticlockwise
    let mut outline: Vec<(f64, f64)> = vec![];
    for (sx, sy, rev) in [(1, 1, false), (-1, 1, true), (-1, -1, false), (1, -1, true)] {
        let mut q: Vec<(i32, i32)> = a.clone();
        if rev {
            q.reverse();
        }
        for (x, y) in q {
            let p = ((sx * x) as f64, (sy * y) as f64);
            if outline.last() != Some(&p) {
                outline.push(p);
            }
        }
    }
    let s = d + 2 * l.wall;
    let ext = s + 4;
    let in_part = |x: f64, y: f64| match l.shape {
        Shape::Full => true,
        Shape::Half => y > 0.0,
        Shape::Quarter => x > 0.0 && y > 0.0,
    };
    let polys: Vec<Vec<(f64, f64)>> = pieces
        .iter()
        .map(|p| p.polygon().into_iter().map(|(x, y)| (x as f64, y as f64)).collect())
        .collect();
    let mut r = Report::default();
    // sample points off the grid lines and off the diagonals
    let (ox, oy) = (0.137, 0.291);
    let n = ((2 * ext) as f64 / step) as i32;
    // bucket pieces by bounding box for speed
    let mut count = vec![0u8; (n * n) as usize];
    for poly in &polys {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for &(x, y) in poly {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
        let i0 = (((x0 + ext as f64 - ox) / step).floor() as i32).max(0);
        let i1 = (((x1 + ext as f64 - ox) / step).ceil() as i32).min(n - 1);
        let j0 = (((y0 + ext as f64 - oy) / step).floor() as i32).max(0);
        let j1 = (((y1 + ext as f64 - oy) / step).ceil() as i32).min(n - 1);
        for i in i0..=i1 {
            for j in j0..=j1 {
                let x = -(ext as f64) + i as f64 * step + ox;
                let y = -(ext as f64) + j as f64 * step + oy;
                if inside_convex(poly, x, y) {
                    let c = &mut count[(i * n + j) as usize];
                    *c = c.saturating_add(1);
                }
            }
        }
    }
    for i in 0..n {
        for j in 0..n {
            let x = -(ext as f64) + i as f64 * step + ox;
            let y = -(ext as f64) + j as f64 * step + oy;
            let c = count[(i * n + j) as usize];
            if c > 1 {
                r.overlaps += 1;
            }
            let disc = inside_convex(&outline, x, y);
            let in_sq = x.abs() < s as f64 && y.abs() < s as f64;
            let want: Option<bool> = match (l.kind, l.inverse) {
                (Kind::Solid, false) => Some(disc && in_part(x, y)),
                (Kind::Solid, true) => Some(!disc && in_sq && in_part(x, y)),
                // rings: only check nothing spills where it shouldn't
                (Kind::Hollow, false) => if !disc || !in_part(x, y) { Some(false) } else { None },
                (Kind::Hollow, true) => if disc || !in_part(x, y) || x.abs() > (d + 2) as f64 || y.abs() > (d + 2) as f64 { Some(false) } else { None },
            };
            match want {
                Some(true) if c == 0 => r.gaps += 1,
                Some(false) if c > 0 => r.spill += 1,
                _ => {}
            }
        }
    }
    r
}
