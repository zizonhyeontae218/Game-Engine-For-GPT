# FlatLand / Maze Chase

A Pac-Man-style playable engine test, authored as schema v2 data and one small Lua hook.
Walls are map tiles; dots, power dots, player and four ghosts are runtime entities.
Arrow keys/WASD or joystick move; queued turns apply at cell intersections.
Collect all food to win, keep away from ghosts, use power dots to eat ghosts temporarily.
Three lives, score and exact save/resume. RESTART begins a fresh maze.

Sprites: platzhersh/pacman-canvas, CC0-1.0. Pinned sources, original SVGs, hashes and
license are in assets/SOURCES.json and assets/LICENSE-CC0.txt. PNGs are rasterizations
of those open-source assets. This demo uses its own maze/rules and synthesized WAVs.
This is a gameplay reproduction for engine testing, not an official Namco release.

Run: `ge4g run examples/flatland_pacman`; test: `ge4g test examples/flatland_pacman`.
Flutter mobile imports the generated flatland-pacman.ge4g; desktop bundles open directly.
