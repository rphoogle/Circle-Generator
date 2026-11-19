//! Writes the v0.3.0 cone test pack using exactly the engine the app uses.
use circlegen::cone::*;
use circlegen::{cone_world, Shape};
fn main() {
    let out = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or("conepack".into()));
    std::fs::create_dir_all(&out).unwrap();
    let b = ConeSettings { shape: Shape::Full, ctype: ConeType::Solid, levels: Levels::Single, bottom_w: 40, bottom_l: 40, top_w: 38, top_l: 38, height: 4, block: 60, band: 15, fill: ConeFill::LargeBlocks, upside_down: false };
    let mut list: Vec<(&str, ConeSettings)> = vec![];
    list.push(("T01_Solid_SingleLevel_40to38_4M", b.clone()));
    list.push(("T02_Hollow_Levels_40to20_20M", ConeSettings { ctype: ConeType::Hollow, levels: Levels::Multiple, top_w: 20, top_l: 20, height: 20, ..b.clone() }));
    list.push(("T03_Solid_Levels_40to20_20M_LargeBlocks", ConeSettings { levels: Levels::Multiple, top_w: 20, top_l: 20, height: 20, ..b.clone() }));
    list.push(("T04_Solid_Levels_40to20_20M_Strips", ConeSettings { levels: Levels::Multiple, top_w: 20, top_l: 20, height: 20, fill: ConeFill::Strips, ..b.clone() }));
    list.push(("T05_Hole_Levels_40to20_10M_block60_LargeBlocks", ConeSettings { ctype: ConeType::Inverse, levels: Levels::Multiple, bottom_w: 20, bottom_l: 20, top_w: 40, top_l: 40, height: 10, ..b.clone() }));
    list.push(("T06_Hole_SingleLevel_30to28_4M_block50", ConeSettings { ctype: ConeType::Inverse, bottom_w: 28, bottom_l: 28, top_w: 30, top_l: 30, height: 4, block: 50, ..b.clone() }));
    list.push(("T07_OvalHole_Levels_50x30to30x10_block70", ConeSettings { ctype: ConeType::Inverse, levels: Levels::Multiple, bottom_w: 30, bottom_l: 10, top_w: 50, top_l: 30, height: 10, block: 70, ..b.clone() }));
    list.push(("T08_SlopedDoughnut_196x113_band15_bottom236x153_10M", ConeSettings { ctype: ConeType::Doughnut, levels: Levels::Multiple, bottom_w: 236, bottom_l: 153, top_w: 196, top_l: 113, height: 10, ..b.clone() }));
    list.push(("T09_Hollow_Levels_40to20_20M_UpsideDown", ConeSettings { ctype: ConeType::Hollow, levels: Levels::Multiple, top_w: 20, top_l: 20, height: 20, upside_down: true, ..b.clone() }));
    list.push(("T10_Half_Solid_Levels_40to20_20M", ConeSettings { shape: Shape::Half, levels: Levels::Multiple, top_w: 20, top_l: 20, height: 20, ..b.clone() }));
    list.push(("T11_Quarter_Hole_Levels_40to20_block60", ConeSettings { shape: Shape::Quarter, ctype: ConeType::Inverse, levels: Levels::Multiple, bottom_w: 20, bottom_l: 20, top_w: 40, top_l: 40, height: 10, ..b.clone() }));
    list.push(("T12_OddSize_Solid_Levels_41to21_20M", ConeSettings { levels: Levels::Multiple, bottom_w: 41, bottom_l: 41, top_w: 21, top_l: 21, height: 20, ..b.clone() }));
    list.push(("T13_Chunky_Hollow_40to20_only5M", ConeSettings { ctype: ConeType::Hollow, levels: Levels::Multiple, top_w: 20, top_l: 20, height: 5, ..b.clone() }));
    list.push(("T14_OvalCone_Solid_Levels_40x24to28x12_12M", ConeSettings { levels: Levels::Multiple, bottom_w: 40, bottom_l: 24, top_w: 28, top_l: 12, height: 12, ..b.clone() }));
    for (name, c) in list {
        match build_cone(&c) {
            Ok(r) => {
                let o = overlaps(&r.bricks);
                let w = cone_world(&r.bricks, (230, 180, 60), "BMC_Plastic", name);
                let p = out.join(format!("{name}.brz"));
                if p.exists() { std::fs::remove_file(&p).unwrap(); }
                w.write_brz(&p).unwrap();
                println!("{name}: {} bricks, {} levels, step {}, top {:?}, overlaps {o}, warnings {:?}", r.bricks.len(), r.levels, r.step, r.top_actual, r.warnings);
            }
            Err(e) => println!("{name}: ERR {e}"),
        }
    }
}
