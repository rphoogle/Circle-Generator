use circlegen::*;
fn main() {
    for band in 1..=4 {
        let mut fails = vec![]; let mut n = 0;
        for w in 6..=300 {
            for len in [w, w * 2, w + 7] {
                n += 1;
                let l = Layout { shape: Shape::Full, kind: Kind::Ring, inverse: false, fill: Fill::Strips, width: w, length: len, hole_w: w - 2*band, hole_l: len - 2*band, wall: 1 };
                if l.hole_w < 2 { continue; }
                if geometry::layout(&l).is_err() { fails.push((w, len)); }
            }
        }
        println!("band {band}: {} of {n} fail; first {:?}", fails.len(), &fails[..fails.len().min(8)]);
    }
}
