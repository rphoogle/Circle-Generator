//! Shell self-test: builds thousands of shells and checks every joint lands
//! each panel where the maths says, with no overlaps and legal brick sizes.
use circlegen::shell::*;
fn main() {
    let mut n = 0;
    let mut bad = 0;
    let mut errs = 0;
    let curves = [Curve::Cone, Curve::Dish, Curve::Sphere, Curve::Custom];
    for &curve in &curves {
        for &(strips, rings) in &[(3u32, 1u32), (4, 2), (7, 3), (12, 11), (24, 8), (40, 16)] {
            for &t in &[1, 2, 3, 5] {
                for &seam in &[Seam::Outside, Seam::Inside, Seam::Middle] {
                    for &spacing in &[Spacing::EqualLength, Spacing::EqualHeight, Spacing::EqualAngle] {
                        for &(bearings, upside, inside, side) in &[
                            (true, false, false, BearingSide::Inside),
                            (true, true, false, BearingSide::Outside),
                            (true, false, true, BearingSide::Inside),
                            (false, true, false, BearingSide::Inside),
                        ] {
                            for variant in 0..3 {
                                let mut s = ShellSettings::default();
                                s.curve = curve;
                                s.strips = strips;
                                s.rings = rings;
                                s.thickness = t;
                                s.seam = seam;
                                s.spacing = spacing;
                                s.bearings = bearings;
                                s.upside_down = upside;
                                s.column_inside = inside;
                                s.bearing_side = side;
                                match variant {
                                    0 => {}
                                    1 => {
                                        s.cone_bottom = 0;
                                        s.cone_top = 90;
                                        s.dish_radius = 120;
                                        s.dish_depth = 200;
                                        s.sphere_input = SphereInput::RadiusDepth;
                                        s.points = vec![(0.0, 0.0), (60.0, 20.0), (80.0, 90.0), (0.0, 150.0)];
                                    }
                                    _ => {
                                        s.cone_bottom = 150;
                                        s.cone_top = 20;
                                        s.dish_radius = 600;
                                        s.dish_depth = 30;
                                        s.start_deg = 30;
                                        s.end_deg = 150;
                                        s.points = vec![(50.0, 0.0), (52.0, 100.0)];
                                    }
                                }
                                n += 1;
                                match build_shell(&s) {
                                    Ok(r) => {
                                        let c = check_shell(&r);
                                        if !c.ok() {
                                            bad += 1;
                                            if bad < 10 {
                                                println!("FAIL {c:?} for {s:?}");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        errs += 1; if std::env::var("SHOW").is_ok() { println!("REFUSED {e}"); }
                                        if e.starts_with("internal") {
                                            bad += 1;
                                            println!("INTERNAL {e} for {s:?}");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{n} shells built, {errs} refused with a message, {bad} problems");
    if bad > 0 {
        std::process::exit(1);
    }
}
