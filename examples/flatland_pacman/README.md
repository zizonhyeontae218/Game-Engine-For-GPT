# FlatLand / Maze Chase

A Pac-Man-style playable engine test, authored as schema v2 data and one small Lua hook.
Walls are map tiles; dots, power dots, player and four ghosts are runtime entities.
Arrow keys/WASD or joystick move; queued turns apply at cell intersections.
Collect all food to win, keep away from ghosts, use power dots to eat ghosts temporarily.
Three lives, score and exact save/resume. RESTART begins a fresh maze.

Animated sprites: bward2/pacman-js, MIT, pinned at 93f1bb08b8e8d4c511e7194b967b3d4fdf765681. Four Pac-Man frames and two ghost frames per direction are extracted unchanged from 16x16 PNG strips. Strip originals, frame rectangles/hashes and MIT license are in assets/animated/.
Original sprites: platzhersh/pacman-canvas, CC0-1.0. Pinned sources, original SVGs, hashes and
license are in assets/SOURCES.json and assets/LICENSE-CC0.txt. PNGs are rasterizations
of those open-source assets. This demo uses its own maze/rules and synthesized WAVs.
This is a gameplay reproduction for engine testing, not an official Namco release.

Run: `ge4g run examples/flatland_pacman`; test: `ge4g test examples/flatland_pacman`.
Flutter mobile imports the generated flatland-pacman.ge4g; desktop bundles open directly.

Mobile presets are authored in controls/layouts.json and bindings.json and automatically travel inside the .ge4g package. Left/right joystick-only presets remove unnecessary action buttons. User-edited profiles remain intact; the game menu can restore the packaged preset. The compact HUD and overlay controls do not reserve half of the game viewport.
