//! Self-check for the cone engine across many settings: no overlaps, no internal errors.
use circlegen::cone::*;
use circlegen::Shape;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let max: i32 = a.get(1).and_then(|v| v.parse().ok()).unwrap_or(40);
    let (mut ok, mut rejected, mut bad) = (0, 0, 0);
    for &ctype in &[ConeType::Solid, ConeType::Hollow, ConeType::Inverse, ConeType::Doughnut] {
        for &levels in &[Levels::Single, Levels::Multiple] {
            for &shape in &[Shape::Full, Shape::Half, Shape::Quarter] {
                for &fill in &[ConeFill::LargeBlocks, ConeFill::Strips] {
                    for &flip in &[false, true] {
                        let mut w = 8;
                        while w <= max {
                            for &(dl, steps, h) in &[(0, 1, 3), (0, 2, 6), (-6, 3, 4), (3, 4, 2), (0, 5, 12), (-1, 1, 1)] {
                                let l = w + dl;
                                if l < 8 { continue; }
                                let (big_w, big_l, small_w, small_l) = (w, l, w - 2 * steps, l - 2 * steps);
                                if small_w < 4 || small_l < 4 { continue; }
                                let (bw, bl, tw, tl) = if ctype == ConeType::Inverse { (small_w, small_l, big_w, big_l) } else { (big_w, big_l, small_w, small_l) };
                                let block = big_w.max(big_l) + 6 + (big_w.max(big_l) % 2 != big_w % 2) as i32;
                                let c = ConeSettings { shape, ctype, levels, bottom_w: bw, bottom_l: bl, top_w: tw, top_l: tl, height: h, block: if big_w % 2 == big_l % 2 { block - (block - big_w) % 2 } else { block }, band: 2.max(small_w / 6), fill, upside_down: flip };
                                match build_cone(&c) {
                                    Ok(r) => {
                                        let o = overlaps(&r.bricks);
                                        if o > 0 {
                                            bad += 1;
                                            if bad < 15 { println!("OVERLAP {o}: {c:?}"); }
                                        } else { ok += 1; }
                                    }
                                    Err(e) => {
                                        if e.starts_with("internal") {
                                            bad += 1;
                                            if bad < 15 { println!("INTERNAL {e}: {c:?}"); }
                                        } else { rejected += 1; }
                                    }
                                }
                            }
                            w += 1;
                        }
                    }
                }
            }
        }
    }
    println!("ok {ok}, rejected with a message {rejected}, problems {bad}");
    if bad > 0 { std::process::exit(1); }
}
