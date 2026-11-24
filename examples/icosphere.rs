//! Icosphere test prefabs (round 2): side wedges, side wedges + micro trim,
//! sweet-spot sizes, a 1000-stud green one, and one held by bearings.
use circlegen::shell::*;
type V3 = [f64; 3];
fn sub(a: V3, b: V3) -> V3 { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn add(a: V3, b: V3) -> V3 { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn sc(a: V3, s: f64) -> V3 { [a[0]*s, a[1]*s, a[2]*s] }
fn dot(a: V3, b: V3) -> f64 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]] }
fn norm3(a: V3) -> f64 { dot(a, a).sqrt() }
fn unit(a: V3) -> V3 { let n = dot(a, a).sqrt(); sc(a, 1.0/n) }

/// Geodesic faces grouped into strips (each strip: neighbours share an edge),
/// plus whether a face touches one of the 12 five-way corners.
fn geodesic(f: usize, r: f64) -> Vec<Vec<([V3; 3], bool)>> {
    let t = (1.0 + 5f64.sqrt()) / 2.0;
    let v: Vec<V3> = [[-1.,t,0.],[1.,t,0.],[-1.,-t,0.],[1.,-t,0.],[0.,-1.,t],[0.,1.,t],[0.,-1.,-t],[0.,1.,-t],[t,0.,-1.],[t,0.,1.],[-t,0.,-1.],[-t,0.,1.]]
        .iter().map(|&p| unit(p)).collect();
    let fi = [[0,11,5],[0,5,1],[0,1,7],[0,7,10],[0,10,11],[1,5,9],[5,11,4],[11,10,2],[10,7,6],[7,1,8],[3,9,4],[3,4,2],[3,2,6],[3,6,8],[3,8,9],[4,9,5],[2,4,11],[6,2,10],[8,6,7],[9,8,1]];
    let mut strips = vec![];
    for [a, b, c] in fi {
        let (a, b, c) = (v[a], v[b], v[c]);
        let p = |i: usize, j: usize| -> V3 {
            let (u, w) = (i as f64 / f as f64, j as f64 / f as f64);
            sc(unit(add(add(sc(a, 1.0-u-w), sc(b, u)), sc(c, w))), r)
        };
        for i in 0..f {
            let mut strip = vec![];
            for j in 0..f - i {
                let corner = (i == 0 && j == 0) || (i == f-1 && j == 0) || (i == 0 && j == f-1);
                strip.push(([p(i, j), p(i+1, j), p(i, j+1)], corner));
                if i + j + 2 <= f { strip.push(([p(i+1, j), p(i+1, j+1), p(i, j+1)], false)); }
            }
            strips.push(strip);
        }
    }
    strips
}

#[derive(Clone, Copy, PartialEq)]
enum Root { Ground, HubSpin, HubFree }

#[derive(Clone, Copy, PartialEq)]
enum Style { Micro, Side, SideTrim, SideNearest }

fn fl(x: f64, step: f64) -> i32 { ((x / step) + 1e-6).floor() as i32 * step as i32 }
fn rd(x: f64, step: f64) -> i32 { (x / step).round() as i32 * step as i32 }

/// Right half of a face: right angle at the origin, leg l along +x, height h along +y.
/// Boxes are (x0, x1, y0, y1, kind, corner); never past the true edge except SideNearest.
fn half(l: f64, h: f64, st: Style) -> Vec<(i32, i32, i32, i32, Kind, u8)> {
    let mut v = vec![];
    match st {
        Style::Micro => {
            let (lm, hm) = (fl(l, 2.0), fl(h, 2.0));
            if lm > 0 && hm > 0 { v.push((0, lm, 0, hm, Kind::Wedge, 0)); }
        }
        Style::Side | Style::SideNearest => {
            let (ls, hs) = if st == Style::Side { (fl(l, 10.0), fl(h, 10.0)) } else { (rd(l, 10.0), rd(h, 10.0)) };
            if ls > 0 && hs > 0 { v.push((0, ls, 0, hs, Kind::SideWedge, 0)); }
        }
        Style::SideTrim => {
            let hm = fl(h, 2.0) as f64;
            let lm = fl(l, 2.0);
            // pick the stud-sized side wedge whose slope matches best (band ≤ 3 studs)
            let mut best: Option<(f64, i32, i32)> = None;
            let top = fl(hm, 10.0);
            for b in [top, top - 10, top - 20, top - 30] {
                if b <= 0 { continue; }
                let w = l * b as f64 / hm; // true width at the band's top
                let a = fl(w, 10.0);
                if a <= 0 { continue; }
                let dev = w - a as f64;
                let score = dev + 0.05 * (hm - b as f64);
                if best.map_or(true, |x| score < x.0) { best = Some((score, a, b)); }
            }
            if let Some((_, a, b)) = best {
                let y0 = hm as i32 - b;
                v.push((0, a, y0, y0 + b, Kind::SideWedge, 0));
                if y0 >= 2 {
                    let ar = fl(l * b as f64 / hm, 2.0).max(a); // rectangle under the side wedge
                    v.push((0, ar, 0, y0, Kind::Micro, 0));
                    if lm > ar { v.push((ar, lm, 0, y0, Kind::Wedge, 0)); }
                }
            } else if lm > 0 && hm > 0.0 {
                v.push((0, lm, 0, hm as i32, Kind::Wedge, 0));
            }
        }
    }
    v
}

fn face_bricks(p: f64, q: f64, h: f64, st: Style) -> Vec<Pb> {
    let mut out = vec![];
    for (left, l) in [(true, p), (false, q)] {
        for (x0, x1, y0, y1, kind, c) in half(l, h, st) {
            let (mut x0, mut x1, mut c) = (x0, x1, c);
            if left { (x0, x1) = (-x1, -x0); c = [1, 0, 3, 2][c as usize]; }
            let (hx, hy) = ((x1 - x0) / 2, (y1 - y0) / 2);
            let size = if kind == Kind::Micro || c % 2 == 0 { (hx, hy, 2) } else { (hy, hx, 2) };
            out.push(Pb { kind, size, pos: ((x0 + x1) / 2, (y0 + y1) / 2, -2), ori: 16 + if kind == Kind::Micro { 0 } else { c } });
        }
    }
    out
}

fn wrap(a: f64) -> f64 { let mut a = a % 360.0; if a > 180.0 { a -= 360.0 } if a <= -180.0 { a += 360.0 } a }

fn build(f: usize, radius: f64, st: Style, bearings: bool, two_tone: bool, root: Root) -> Result<(ShellResult, Vec<f64>), String> {
    let mut apex: Vec<f64> = vec![];
    let strips = geodesic(f, radius);
    let mut panels: Vec<Panel> = vec![];
    let mut main = vec![];
    let mut column = vec![];
    const I3: [[f64; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
    let hubbed = bearings && root != Root::Ground;
    let n = if hubbed { strips.iter().map(|s| s.len()).sum::<usize>() as i32 } else { strips.len() as i32 };
    let lowest = -n; // stack centred on the middle of the sphere
    if bearings && root == Root::Ground {
        main.push(Pb { kind: Kind::Micro, size: (3, 3, 1), pos: (0, 0, lowest - 2), ori: 16 });
        for k in 0..n { column.push(main.len()); main.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (0, 0, lowest + 2 * k), ori: 16 }); }
    }
    if bearings && root != Root::Ground {
        // hub entity in the middle carries the strip bearings, so everything moves as one
        let mut hb = vec![Pb { kind: Kind::Micro, size: (3, 3, 1), pos: (0, 0, lowest - 2), ori: 16 }];
        for k in 0..n { hb.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (0, 0, lowest + 2 * k), ori: 16 }); }
        let mut hub = Panel { ring: 0, strip: 0, loc: [0.0; 3], rot: I3, bricks: hb, colour: 0, length: 0.0, w_in: 0.0, w_out: 0.0, joint: None };
        apex.push(0.0);
        if root == Root::HubSpin {
            // one bearing on the ground under the sphere turns the hub (and the whole sphere) on the upright axis
            let zb = -(radius.ceil() as i32) - 14;
            main.push(Pb { kind: Kind::Micro, size: (5, 5, 1), pos: (0, 0, zb - 2), ori: 16 });
            column.push(main.len());
            main.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (0, 0, zb), ori: 16 });
            let (off, q) = joint_for(([0.0; 3], I3), [0.0, 0.0, zb as f64], &d_of(16), 0.0, (hub.loc, hub.rot));
            hub.joint = Some(Joint { holder: Holder::Column(0), angle: 0.0, off, q });
        }
        panels.push(hub);
    }
    let use_hub = bearings && root != Root::Ground;
    for (si, strip) in strips.iter().enumerate() {
        let first = panels.len();
        for (k, (tri, corner)) in strip.iter().enumerate() {
            let centroid = sc(add(add(tri[0], tri[1]), tri[2]), 1.0/3.0);
            // base edge: the edge shared with the next face (with bearings), else the longest
            let next = if bearings && root == Root::Ground { strip.get(k + 1).map(|x| x.0) } else { None };
            let mut base: Option<(V3, V3, V3)> = None;
            if let Some(n) = next {
                for e in 0..3 {
                    let (b, c, a) = (tri[e], tri[(e+1)%3], tri[(e+2)%3]);
                    let near = |x: V3| n.iter().any(|y| dot(sub(*y, x), sub(*y, x)) < 1e-6);
                    if near(b) && near(c) { base = Some((b, c, a)); }
                }
            }
            let (mut b, mut c, a) = base.unwrap_or_else(|| {
                // micro: longest edge; side wedges: the edge whose legs land nearest whole studs
                let mut best = (tri[0], tri[1], tri[2]);
                let mut bs = f64::MAX;
                for e in 0..3 {
                    let (b, c, a) = (tri[e], tri[(e+1)%3], tri[(e+2)%3]);
                    let x = unit(sub(c, b));
                    let ft = add(b, sc(x, dot(sub(a, b), x)));
                    let (p, q, h) = (dot(sub(ft, b), x), dot(sub(c, ft), x), norm3(sub(a, ft)));
                    if p <= 0.0 || q <= 0.0 { continue; }
                    let score = if st == Style::Micro { -dot(sub(c, b), sub(c, b)) } else {
                        [p, q, h].iter().map(|v| (v - (v / 10.0).round() * 10.0).abs()).fold(0.0, f64::max)
                    };
                    if score < bs { bs = score; best = (b, c, a); }
                }
                best
            });
            if dot(cross(sub(c, b), sub(a, b)), centroid) < 0.0 { std::mem::swap(&mut b, &mut c); }
            let x = unit(sub(c, b));
            let foot = add(b, sc(x, dot(sub(a, b), x)));
            let y = unit(sub(a, foot));
            let z = cross(x, y);
            let (p, q, h) = (dot(sub(foot, b), x), dot(sub(c, foot), x), dot(sub(a, foot), y));
            if p <= 0.0 || q <= 0.0 { return Err("a face's corner is not over its base".into()); }
            let mut bricks = face_bricks(p, q, h, st);
            // slide the (rounded-down) bricks along the base so both base corners get the same gap
            let step = match st { Style::Micro => 2.0, _ => 10.0 };
            let shift = if st == Style::Micro || st == Style::Side {
                let fl = |v: f64| (v / step + 1e-6).floor() * step;
                let (pf, qf, hf) = (fl(p), fl(q), fl(h));
                // keep all three brick corners inside the true triangle; take the middle of what's allowed
                let k = 1.0 - hf / h;
                let lo = (pf - p).max(-p * k);
                let hi = (q - qf).min(q * k);
                if hi >= lo { (lo + hi) / 2.0 } else { 0.0 }
            } else { 0.0 };
            let foot_brick = add(foot, sc(x, shift));
            apex.push(-shift);
            if next.is_some() {
                // hinge bearing on the outer face, centred on the shared edge: it turns on that edge
                let xm = ((q - p) / 2.0).round() as i32;
                bricks.push(Pb { kind: Kind::Bearing, size: (1, 1, 1), pos: (xm, 0, 1), ori: 0 });
            }
            let rot = [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]];
            panels.push(Panel { ring: k, strip: si, loc: foot_brick, rot, bricks, colour: if two_tone && *corner { 1 } else { 0 },
                length: h, w_in: p + q, w_out: p + shift, joint: None }); // w_out = left leg from brick origin (test only)
        }
        if bearings {
            for idx in first..panels.len() {
                let child = (panels[idx].loc, panels[idx].rot);
                let j = if use_hub {
                    let k = idx - 1; // every face on its own hub bearing
                    let b = panels[0].bricks.iter().filter(|b| b.kind == Kind::Bearing).nth(k).copied().unwrap();
                    let (off, q) = joint_for(([0.0; 3], I3), [b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64], &d_of(16), 0.0, child);
                    Joint { holder: Holder::PanelK(0, k), angle: 0.0, off, q }
                } else if idx == first {
                    let b = main[column[si]];
                    let (off, q) = joint_for(([0.0; 3], [[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]]), [b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64], &d_of(16), 0.0, child);
                    Joint { holder: Holder::Column(si), angle: 0.0, off, q }
                } else {
                    let pi = idx - 1;
                    let parent = (panels[pi].loc, panels[pi].rot);
                    let b = *panels[pi].bricks.iter().find(|b| b.kind == Kind::Bearing).unwrap();
                    let rel = mm(&tr(&parent.1), &child.1);
                    let n = [rel[0][2], rel[1][2], rel[2][2]];
                    let angle = wrap(n[1].atan2(n[2]).to_degrees());
                    let (off, q) = joint_for(parent, [b.pos.0 as f64, b.pos.1 as f64, b.pos.2 as f64], &d_of(b.ori), angle, child);
                    Joint { holder: Holder::Panel(pi), angle, off, q }
                };
                panels[idx].joint = Some(j);
            }
        }
    }
    Ok((ShellResult { panels, main, column, outline: vec![], rings_at: vec![], warnings: vec![], inside_sign: -1.0 }, apex))
}

fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or("icopack".into()));
    std::fs::create_dir_all(&out).unwrap();
    let white = [255u8, 255, 255];
    let red = [224u8, 85, 75];
    let green = [60u8, 160, 70];
    // (name, detail, diameter in studs, style, bearings, two-tone, colours, hide bearings)
    let list = [
        ("I12_Icosphere_71studs_80faces_SideWedges_Spin", 2, 71.2, Style::Side, true, true, [white, red], false, Root::HubSpin),
        ("I13_Icosphere_71studs_80faces_SideWedges_Roll", 2, 71.2, Style::Side, true, true, [white, red], true, Root::HubFree),
        ("I14_Icosphere_317studs_180faces_SideWedges_Spin", 3, 317.2, Style::Side, true, true, [white, red], false, Root::HubSpin),
    ];
    for (name, f, d, st, bear, two, cols, hide, root) in list {
        let (res, apex) = match build(f, d * 5.0, st, bear, two, root) { Ok(r) => r, Err(e) => { println!("{name}: ERR {e}"); continue; } };
        let chk = check_shell(&res);
        // accuracy in each face's own plane: corners outside the true triangle (clipping)
        // and how far each true corner is from the nearest brick corner (gaps)
        let (mut poke, mut gap) = (0.0f64, 0.0f64);
        for (pi, p) in res.panels.iter().enumerate() {
            if p.length == 0.0 { continue; } // the hub
            let (pl, ql, h) = (p.w_out, p.w_in - p.w_out, p.length);
            let tri = [(-pl, 0.0), (ql, 0.0), (apex[pi], h)];
            let mut pts: Vec<(f64, f64)> = vec![];
            for b in &p.bricks {
                if b.kind == Kind::Bearing { continue; }
                let (hx, hy, _) = b.half();
                let (cx, cy) = (b.pos.0 as f64, b.pos.1 as f64);
                let all = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
                for (i, (u, v)) in all.iter().enumerate() {
                    // a wedge has no corner opposite its right angle
                    if b.kind != Kind::Micro && i == ((b.ori & 3) as usize + 2) % 4 { continue; }
                    pts.push((cx + u * hx as f64, cy + v * hy as f64));
                }
            }
            for &(x, y) in &pts {
                // distance outside the triangle (0 if inside)
                let mut out: f64 = (-y).max(0.0);
                for (a, b) in [(tri[0], tri[2]), (tri[2], tri[1])] {
                    let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                    let n = (ex * ex + ey * ey).sqrt();
                    let side = (ex * (y - a.1) - ey * (x - a.0)) / n; // + = left of a->b
                    out = out.max(side);
                }
                poke = poke.max(out);
            }
            for t in tri {
                let d = pts.iter().map(|q| ((q.0 - t.0).powi(2) + (q.1 - t.1).powi(2)).sqrt()).fold(f64::MAX, f64::min);
                gap = gap.max(d);
            }
        }
        println!("   biggest overhang past a true edge: {:.2} units; biggest corner gap: {:.1} units ({:.2} studs)", poke, gap, gap / 10.0);
        let s = ShellSettings { bearings: bear, hide_bearings: hide, colours: cols.to_vec(), ..ShellSettings::default() };
        let w = shell_world(&s, &res, name);
        std::fs::write(out.join(format!("{name}.brz")), w.to_brz_vec().unwrap()).unwrap();
        let kinds = |k: Kind| res.panels.iter().flat_map(|p| &p.bricks).filter(|b| b.kind == k).count();
        println!("{name}: {} entities, {} bricks (side {} micro-wedge {} micro {} bearings {}), ok={} {:?}",
            res.panels.len(), res.brick_count(), kinds(Kind::SideWedge), kinds(Kind::Wedge), kinds(Kind::Micro), res.bearing_count(), chk.ok(), chk);
    }
}
