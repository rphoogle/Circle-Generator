//! Icosphere (geodesic sphere) for the Shell tab.
//!
//! In plain English: an icosahedron (20 triangles) has each triangle cut into
//! smaller triangles and pushed out onto a sphere. Every small triangle is a
//! flat face on its own entity, built from just two wedges: the triangle is
//! split down its height into two right-angled halves.
//!
//! Bricks only come in whole micros (micro wedges) or whole studs (Side
//! Wedges), so each face is rounded DOWN and then slid along its base to the
//! middle of the space it is allowed, so nothing ever sticks past a true edge
//! (testers asked for no clipping); any rounding shows as a small gap. Some
//! sizes ("sweet spots") make every face land almost exactly on whole studs.
//!
//! Movement: Frozen (no bearings), Spin (a hub entity in the middle holds every
//! face on its own bearing; the hub sits on one ground bearing that turns the
//! whole ball) or Free (same hub, not attached: unfreeze it to roll).

use crate::shell::*;

type V3 = [f64; 3];
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sc(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn unit(a: V3) -> V3 {
    let n = dot(a, a).sqrt();
    sc(a, 1.0 / n)
}
const I3: M = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

pub const MAX_DETAIL: u32 = 6;

pub fn face_count(detail: u32) -> usize {
    20 * (detail * detail) as usize
}

/// Faces of a geodesic sphere of radius 1 (frequency f), and whether each face
/// touches one of the 12 five-way corners.
pub fn faces(f: usize) -> Vec<([V3; 3], bool)> {
    let t = (1.0 + 5f64.sqrt()) / 2.0;
    let v: Vec<V3> = [
        [-1., t, 0.], [1., t, 0.], [-1., -t, 0.], [1., -t, 0.], [0., -1., t], [0., 1., t],
        [0., -1., -t], [0., 1., -t], [t, 0., -1.], [t, 0., 1.], [-t, 0., -1.], [-t, 0., 1.],
    ]
    .iter()
    .map(|&p| unit(p))
    .collect();
    let fi = [
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11], [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9], [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];
    let mut out = vec![];
    for [a, b, c] in fi {
        let (a, b, c) = (v[a], v[b], v[c]);
        let p = |i: usize, j: usize| -> V3 {
            let (u, w) = (i as f64 / f as f64, j as f64 / f as f64);
            unit(add(add(sc(a, 1.0 - u - w), sc(b, u)), sc(c, w)))
        };
        for i in 0..f {
            for j in 0..f - i {
                let corner = (i == 0 && j == 0) || (i == f - 1 && j == 0) || (i == 0 && j == f - 1);
                out.push(([p(i, j), p(i + 1, j), p(i, j + 1)], corner));
                if i + j + 2 <= f {
                    out.push(([p(i + 1, j), p(i + 1, j + 1), p(i, j + 1)], false));
                }
            }
        }
    }
    out
}

/// One way of splitting a face: legs p (left), q (right) along the base, height h.
#[derive(Clone, Copy, Debug)]
struct Split {
    p: f64,
    q: f64,
    h: f64,
}

/// Rounded-down sizes, the slide along the base, and the worst corner gap.
fn fit_split(sp: Split, step: f64) -> (f64, f64, f64, f64, f64) {
    let fl = |v: f64| (v / step + 1e-9).floor() * step;
    let (pf, qf, hf) = (fl(sp.p), fl(sp.q), fl(sp.h));
    let k = 1.0 - hf / sp.h;
    let lo = (pf - sp.p).max(-sp.p * k);
    let hi = (sp.q - qf).min(sp.q * k);
    let shift = if hi >= lo { (lo + hi) / 2.0 } else { 0.0 };
    // brick corners (after the slide) against the true corners
    let d = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let gap = d((-pf + shift, 0.0), (-sp.p, 0.0)).max(d((qf + shift, 0.0), (sp.q, 0.0))).max(d((shift, hf), (0.0, sp.h)));
    (pf, qf, hf, shift, gap)
}

fn splits(tri: &[V3; 3]) -> Vec<(usize, Split)> {
    let mut v = vec![];
    for e in 0..3 {
        let (b, c, a) = (tri[e], tri[(e + 1) % 3], tri[(e + 2) % 3]);
        let x = unit(sub(c, b));
        let foot = add(b, sc(x, dot(sub(a, b), x)));
        let (p, q) = (dot(sub(foot, b), x), dot(sub(c, foot), x));
        let h = dot(sub(a, foot), sub(a, foot)).sqrt();
        if p > 1e-9 && q > 1e-9 {
            v.push((e, Split { p, q, h }));
        }
    }
    v
}

/// The worst corner gap (studs) of a whole sphere at diameter `d` studs.
fn sphere_gap(shapes: &[Vec<Split>], d: f64, step_studs: f64) -> f64 {
    let r = d / 2.0;
    let mut worst: f64 = 0.0;
    for opts in shapes {
        let best = opts
            .iter()
            .map(|sp| fit_split(Split { p: sp.p * r, q: sp.q * r, h: sp.h * r }, step_studs).4)
            .fold(f64::MAX, f64::min);
        worst = worst.max(best);
    }
    worst
}

fn unique_shapes(detail: u32) -> Vec<Vec<Split>> {
    let mut seen: Vec<(Vec<i64>, Vec<Split>)> = vec![];
    for (tri, _) in faces(detail as usize) {
        let s: Vec<Split> = splits(&tri).into_iter().map(|x| x.1).collect();
        let mut key: Vec<i64> = s.iter().flat_map(|x| [(x.p * 1e7) as i64, (x.q * 1e7) as i64, (x.h * 1e7) as i64]).collect();
        key.sort();
        if !seen.iter().any(|k| k.0 == key) {
            seen.push((key, s));
        }
    }
    seen.into_iter().map(|x| x.1).collect()
}

/// Best diameters (studs) between `lo` and `hi`, with their worst gap (studs).
pub fn best_sizes(detail: u32, brick: IcoBrick, lo: f64, hi: f64, count: usize) -> Vec<(f64, f64)> {
    let shapes = unique_shapes(detail.clamp(1, MAX_DETAIL));
    let step = if brick == IcoBrick::Side { 1.0 } else { 0.2 };
    let mut all: Vec<(f64, f64)> = vec![];
    let mut d = lo;
    while d <= hi {
        all.push((d, sphere_gap(&shapes, d, step)));
        d += 0.02;
    }
    all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let mut out: Vec<(f64, f64)> = vec![];
    for (d, g) in all {
        if out.iter().all(|o| (o.0 - d).abs() > 2.0) {
            out.push(((d * 100.0).round() / 100.0, g));
            if out.len() == count {
                break;
            }
        }
    }
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    out
}

/// Worst corner gap (studs) for the current settings.
pub fn current_gap(s: &ShellSettings) -> f64 {
    let step = if s.ico_brick == IcoBrick::Side { 1.0 } else { 0.2 };
    sphere_gap(&unique_shapes(s.ico_detail.clamp(1, MAX_DETAIL)), s.ico_diameter, step)
}

pub fn build_icosphere(s: &ShellSettings) -> Result<ShellResult, String> {
    if !(1..=MAX_DETAIL).contains(&s.ico_detail) {
        return Err("Detail must be between 20 and 720 faces.".into());
    }
    if !(s.ico_diameter >= 2.0 && s.ico_diameter <= 4000.0) {
        return Err("The diameter must be between 2 and 4000 studs.".into());
    }
    if s.colours.is_empty() {
        return Err("Pick at least one colour.".into());
    }
    let side = s.ico_brick == IcoBrick::Side;
    let (step, half_t) = if side {
        if !(1..=20).contains(&s.ico_plates) {
            return Err("Thickness must be 1 to 20 plates.".into());
        }
        (10.0, 2 * s.ico_plates)
    } else {
        if !(1..=40).contains(&s.thickness) {
            return Err("Thickness must be 1 to 40 micro bricks.".into());
        }
        (2.0, s.thickness)
    };
    let radius = s.ico_diameter * 5.0; // units
    let kind = if side { Kind::SideWedge } else { Kind::Wedge };
    let hubbed = s.movement != Movement::Frozen;
    let all = faces(s.ico_detail as usize);
    let n = all.len() as i32;
    let mut panels: Vec<Panel> = vec![];
    let mut main: Vec<Pb> = vec![];
    let mut column: Vec<usize> = vec![];
    let mut worst_gap: f64 = 0.0;

    if hubbed {
        let lowest = -n; // bearing stack centred on the middle of the sphere
        let mut hb = vec![Pb { kind: Kind::Micro, size: (3, 3, 1), pos: (0, 0, lowest - 2), ori: 16 }];
        for k in 0..n {
            hb.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (0, 0, lowest + 2 * k), ori: 16 });
        }
        let mut hub = Panel {
            ring: 0,
            strip: 0,
            loc: [0.0; 3],
            rot: I3,
            bricks: hb,
            colour: 0,
            length: 0.0,
            w_in: 0.0,
            w_out: 0.0,
            joint: None,
            shape: vec![],
        };
        if s.movement == Movement::Spin {
            let zb = -(radius.ceil() as i32) - 2 * half_t - 12;
            main.push(Pb { kind: Kind::Micro, size: (5, 5, 1), pos: (0, 0, zb - 2), ori: 16 });
            column.push(main.len());
            main.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (0, 0, zb), ori: 16 });
            let (off, q) = joint_for(([0.0; 3], I3), [0.0, 0.0, zb as f64], &d_of(16), 0.0, (hub.loc, hub.rot));
            hub.joint = Some(Joint { holder: Holder::Column(0), angle: 0.0, off, q });
        }
        panels.push(hub);
    }

    for (k, (tri0, corner)) in all.iter().enumerate() {
        let tri = [sc(tri0[0], radius), sc(tri0[1], radius), sc(tri0[2], radius)];
        let centroid = sc(add(add(tri[0], tri[1]), tri[2]), 1.0 / 3.0);
        // the split whose rounding leaves the smallest gap
        let (e, sp, fit) = splits(&tri)
            .into_iter()
            .map(|(e, sp)| (e, sp, fit_split(sp, step)))
            .min_by(|a, b| a.2 .4.partial_cmp(&b.2 .4).unwrap())
            .ok_or("internal: a face has no usable base")?;
        let (pf, qf, hf, _, gap) = fit;
        if pf < step - 1e-9 || qf < step - 1e-9 || hf < step - 1e-9 {
            return Err(format!(
                "At {} studs the faces are too small for {}. Make it bigger or use less detail.",
                s.ico_diameter,
                if side { "Side Wedges" } else { "micro wedges" }
            ));
        }
        worst_gap = worst_gap.max(gap);
        let (mut b, mut c, a) = (tri[e], tri[(e + 1) % 3], tri[(e + 2) % 3]);
        let mut sp = sp;
        // outward-facing: z = x × y must point away from the centre
        if dot(cross(sub(c, b), sub(a, b)), centroid) < 0.0 {
            std::mem::swap(&mut b, &mut c);
            sp = Split { p: sp.q, q: sp.p, h: sp.h };
        }
        // sizes and slide for the (possibly mirrored) split
        let (pf, qf, hf, shift, _) = fit_split(sp, step);
        let x = unit(sub(c, b));
        let foot = add(b, sc(x, dot(sub(a, b), x)));
        let y = unit(sub(a, foot));
        let z = cross(x, y);
        let (pi, qi, hi) = (pf.round() as i32, qf.round() as i32, hf.round() as i32);
        // two right-angled wedges, outer face on the true plane (z = 0), body inward
        let bricks = vec![
            Pb { kind, size: (hi / 2, pi / 2, half_t), pos: (-pi / 2, hi / 2, -half_t), ori: 16 + 1 },
            Pb { kind, size: (qi / 2, hi / 2, half_t), pos: (qi / 2, hi / 2, -half_t), ori: 16 },
        ];
        let rot = [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]];
        let loc = add(foot, sc(x, shift));
        let colour = if s.ico_two_tone && *corner { 1 } else { 0 };
        let mut panel = Panel {
            ring: 0,
            strip: k,
            loc,
            rot,
            bricks,
            colour,
            length: sp.h,
            w_in: sp.p + sp.q,
            w_out: 0.0,
            joint: None,
            shape: vec![(-sp.p - shift, 0.0), (sp.q - shift, 0.0), (-shift, sp.h)],
        };
        if hubbed {
            let bb = panels[0].bricks.iter().filter(|b| b.kind == Kind::Bearing).nth(k).copied().unwrap();
            let (off, q) = joint_for(([0.0; 3], I3), [bb.pos.0 as f64, bb.pos.1 as f64, bb.pos.2 as f64], &d_of(16), 0.0, (loc, rot));
            panel.joint = Some(Joint { holder: Holder::PanelK(0, k), angle: 0.0, off, q });
        }
        panels.push(panel);
    }
    let mut warnings = vec![];
    if side && worst_gap / 10.0 > 0.3 {
        warnings.push(format!(
            "Side Wedges leave gaps of up to {:.2} stud at this size; try one of the best sizes.",
            worst_gap / 10.0
        ));
    }
    Ok(ShellResult {
        panels,
        main,
        column,
        outline: vec![],
        rings_at: vec![],
        warnings,
        inside_sign: -1.0,
        gap: worst_gap,
    })
}
