//! Self-test: builds thousands of layouts and checks each for gaps and overlaps.
use circlegen::*;
fn thin_ok() {}
fn run(l: Layout, bad: &mut usize, total: &mut usize) {
    let s = Settings { layout: l, style: BrickStyle::Micro, height: 6, color: (255, 255, 255), material: "BMC_Plastic".into(), name: "t".into() };
    *total += 1;
    if validate(&s).is_err() { *total -= 1; return; }
    match build(&s) {
        Err(e) if e.contains("too thin") => { thin_ok(); }
        Err(e) => { *bad += 1; if *bad < 15 { println!("BUILD ERR {l:?}: {e}"); } }
        Ok(b) => {
            let r = check::check(&l, &b.pieces);
            if !r.ok() { *bad += 1; if *bad < 15 { println!("CHECK {l:?}: {r:?}"); } }
        }
    }
}
fn main() {
    let a: Vec<i32> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (mind, maxd, step) = (a[0], a[1], a[2]);
    let (mut bad, mut total) = (0, 0);
    let shapes = [Shape::Full, Shape::Half, Shape::Quarter];
    for w in (mind..=maxd).step_by(step as usize) {
        // circles plus a few oval proportions
        let lens = [w, w + 1, w * 2, (w * 3) / 2 + 3];
        for &len in &lens {
            for shape in shapes {
                for (kind, inv) in [(Kind::Solid, false), (Kind::Hollow, false), (Kind::Solid, true), (Kind::Hollow, true)] {
                    for fill in [Fill::Strips, Fill::Blocks] {
                        if kind == Kind::Hollow && fill == Fill::Blocks { continue; }
                        let l = Layout { shape, kind, inverse: inv, fill, width: w, length: len, hole_w: 0, hole_l: 0, wall: 1 + w % 4 };
                        run(l, &mut bad, &mut total);
                    }
                }
                // doughnuts: several band thicknesses
                for band in [1, 2, 3, 5, w / 4, w / 3] {
                    if band < 1 || w - 2 * band < 2 || len - 2 * band < 2 { continue; }
                    for inv in [false, true] {
                        for fill in [Fill::Strips, Fill::Blocks] {
                            let l = Layout { shape, kind: Kind::Ring, inverse: inv, fill, width: w, length: len, hole_w: w - 2 * band, hole_l: len - 2 * band, wall: 2 };
                            run(l, &mut bad, &mut total);
                        }
                    }
                }
            }
        }
    }
    println!("{total} layouts checked, {bad} problems");
    for (k, i, f) in [(Kind::Solid, false, Fill::Blocks), (Kind::Solid, false, Fill::Strips), (Kind::Hollow, false, Fill::Strips)] {
        let l = Layout::circle(Shape::Full, k, i, f, 80, 5);
        let s = Settings { layout: l, style: BrickStyle::Micro, height: 18, color: (255, 255, 255), material: "BMC_Plastic".into(), name: "t".into() };
        let b = build(&s).unwrap();
        println!("80M {k:?} {f:?}: {} bricks", b.bricks.len());
    }
}
