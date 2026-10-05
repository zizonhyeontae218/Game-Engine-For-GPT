# FlatLand 0.2 / smallest useful authoring context

Rust owns the simulation; JSON5 handles ordinary gameplay, Lua 5.4 extensions.
Start from `examples/flatland_signal_yard`. Each entity placement can inherit one prefab.
Map cells are walls/floors; entity bodies use `flatland.body: pass|fixed|push`.
Scene `resources` reference JSON5 gameplay/prefabs/sounds/rules definitions.
Declare required features in ge4g.toml; pack_game emits bundle schema 3 for feature games.

```sh
ge4g capabilities --json
ge4g schema attack --json
ge4g resource PROJECT data/systems.json5 gameplay/attacks/blade --json
ge4g patch PROJECT data/systems.json5 gameplay/attacks/blade --expected REVISION --patch change.json --json
ge4g test PROJECT --json
```

A patch file is a small object, e.g. `{"damage":4}`. Objects merge; arrays replace.
Entity resource paths use stable IDs (`entities/guard`), not array indices.
Revision conflicts or validation errors leave source bytes untouched. Inspect again to repair.
Query one component/item/event/prefab; full scene/schema expansion is opt-in.

Common action ops: give/take/use/equip, quest/objective, attack/heal/damage,
map_patch/plane/elevate, play_clip/animate, event_scene/music/sound, set/add/roll.
Conditions: state, timer, has_item, quest_is, objectives_complete, all/any/not.
Event instructions: do, wait, say, choice, branch, jump, call, camera, battle, return.
Use schema queries for precise fields/defaults. Saveable waits use the event controller,
never a Lua stack. Replay schema 2 records named inputs and choice/do/skip commands.
Desktop bundles include Flutter and Rust. Mobile imports data-only .ge4g packages.
