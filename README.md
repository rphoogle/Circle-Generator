# Circle Generator for Brickadia

Makes smooth circles, cones, curved shells and icospheres as Brickadia
prefabs (.brz). Built on [brdb](https://github.com/brickadia-community/brdb).

- **Circle** – full / half / quarter circles and ovals; solid, hollow, doughnut and inverse (round holes); micro bricks or studs (Default, Tile, Smooth).
- **Cone** – sloped circles and ovals, funnels, sloped doughnuts, cone-shaped holes.
- **Shell** – domes, parabolic dishes, spheres and custom outlines (curve editor), built from flat entity panels, optionally on bearing hinges.
- **Icosphere** – geodesic spheres (20–720 faces) in micro wedges or Side Wedges, with best-fit sizes; frozen, spinning or free to roll.

## Download
Get the latest **CircleGenerator.exe** from the
[Releases page](https://github.com/rphoogle/Circle-Generator/releases/latest).
No install needed. Windows may show a SmartScreen warning because the program
is unsigned: choose *More info* then *Run anyway*.

(Every push also builds a test .exe under the **Actions** tab → latest run →
*Artifacts* → **CircleGenerator-windows**; you need to be signed in to GitHub.)

## How it is organised
- `src/geometry.rs` – where every brick goes (the shape rules)
- `src/lib.rs` – turns the layout into bricks and a .brz prefab
- `src/check.rs` – self-check for gaps and overlaps, run before every save
- `src/cone.rs`, `src/shell.rs`, `src/icosphere.rs` – the Cone, Shell and Icosphere tabs
- `src/main.rs` – the window
- `src/bin/sweep.rs`, `src/bin/testpack.rs` – test tools

Units: 1 M = one micro brick (0.2 stud); 5 M = 1 stud; 2 M = 1 plate; 6 M = 1 brick.
Largest single brick: 800 studs (4000 M). Fonts: DejaVu (see assets/DejaVu-LICENSE.txt).
