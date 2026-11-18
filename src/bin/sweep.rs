use circlegen::*;
fn main() {
    let a: Vec<i32> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (mind, maxd, step) = (a[0], a[1], a[2]);
    let mut bad = 0; let mut total = 0;
    for d in (mind..=maxd).step_by(step as usize) {
        for shape in [Shape::Full, Shape::Half, Shape::Quarter] {
            if shape != Shape::Full && d % 2 != 0 { continue; }
            for (kind, inv) in [(Kind::Solid,false),(Kind::Hollow,false),(Kind::Solid,true),(Kind::Hollow,true)] {
                for fill in [Fill::Strips, Fill::Blocks] {
                    if kind == Kind::Hollow && fill == Fill::Blocks { continue; }
                    for wall in [1, 3] {
                        if !(kind == Kind::Solid && inv) && wall == 3 { continue; }
                        let l = Layout { shape, kind, inverse: inv, fill, diameter: d, wall };
                        let s = Settings { layout: l, style: BrickStyle::Micro, height: 6, color: (255,255,255), material: "BMC_Plastic".into(), name: "t".into() };
                        total += 1;
                        match build(&s) {
                            Err(e) => { bad += 1; if bad < 15 { println!("BUILD ERR d={d} {shape:?} {kind:?} inv={inv} {fill:?}: {e}"); } }
                            Ok(b) => {
                                let r = check::check(&l, &b.pieces);
                                if !r.ok() { bad += 1; if bad < 15 { println!("CHECK d={d} {shape:?} {kind:?} inv={inv} {fill:?} w={wall}: {r:?}"); } }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{total} layouts checked, {bad} problems");
    for (k,i,f) in [(Kind::Solid,false,Fill::Blocks),(Kind::Solid,false,Fill::Strips),(Kind::Hollow,false,Fill::Strips)] {
        let l = Layout { shape: Shape::Full, kind: k, inverse: i, fill: f, diameter: 80, wall: 5 };
        let s = Settings { layout: l, style: BrickStyle::Micro, height: 18, color: (255,255,255), material: "BMC_Plastic".into(), name: "t".into() };
        let b = build(&s).unwrap(); println!("80M {k:?} {f:?}: {} bricks", b.bricks.len());
    }
}
