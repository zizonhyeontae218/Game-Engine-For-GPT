# P0-S read-only inventory

Observed main HEAD: `293ba513f5727d4a7a0a59476c9a98eb193aaa25`, initially clean. Origin: https://github.com/zizonhyeontae218/Game-Engine-For-GPT.git. Scout performed no edits, installations, builds or tests.

## Actual language/build/modules

Root Cargo workspace: Rust2024, resolver3, version0.2.0, seven crates. `client/pubspec.yaml`: Flutter/Dart client0.2.0+8, Dart SDK >=3.13.0 <4.0.0; local `packages/ge4g_native`.

| Module | Observed responsibilities and dependencies |
|---|---|
| core | common integer math/input/state/events/snapshots; no engine crate dependencies |
| project | filesystem/data/reference validation, TOML/JSON5, PNG; depends core |
| runtime | World/tick/scene/save/replay, FlatLand/combat/view state; core/project/mlua Lua5.4; render2d only dev-dependency |
| render2d | CPU Frame/render/anchors/PNG; core/project |
| platform | minifb window/input, rodio audio; core/project/runtime/render2d; optional window feature |
| client | cdylib/staticlib/rlib, C/JSON session/frame ABI; core/project/runtime/render2d |
| cli | clap CLI, JSON discovery/schema/replay; interactive default, no-default-features headless |

Evidence: `Cargo.toml`, `crates/*/Cargo.toml`, `client/pubspec.yaml`, `docs/ARCHITECTURE.md`.

## Current API evidence

- `crates/ge4g-core/src/lib.rs:94`: Input/direction/cardinal/interact/axes; :145 StateStore; :202 Event(tick/kind/scene/entity/JSON data); :212 EntitySnapshot(color/texture/layer/blocking/trigger/flatland); :229 Snapshot(camera/background/entities/state/events/flatland).
- `crates/ge4g-project/src/lib.rs:108`: Entity(sprite/collider/player/trigger/interaction/FlatLand); :138 Scene(camera/map/rules/script/resources/gameplay); :317 Project(filesystem root/scenes/decoded textures/scripts).
- `crates/ge4g-runtime/src/lib.rs:77`: World exposes Project and FlatLand state. :258 snapshot; :262 render_snapshot; :410 step_actions; :448 step_replay; :467 release_inputs; :475 step(schema2→step_flatland, legacy movement/collision); :622 save; :686 snapshot_hash. Resume/ResumeError reexports.
- `crates/ge4g-runtime/src/gameplay.rs:43`: Systems includes RNG, :103 seeded initialization. Existing choose/command/skip_event_wait are gameplay behavior, not new plugin lifecycle.
- `crates/ge4g-runtime/src/view.rs:6`: GameplayView uses project gameplay View.
- `crates/ge4g-render2d/src/lib.rs:77`: render requires Project/Snapshot.
- `crates/ge4g-client/src/lib.rs:16`: ABI_VERSION1; :27 OnceLock<Mutex<Registry>>; :207 request_json. Header exports ge4g_abi_version/ge4g_request_json/ge4g_free_string/ge4g_frame_copy.
- `crates/ge4g-cli/src/main.rs:266`: discovery game schemas1/2, bundle schemas1/2/3, ABI1, capabilities/query/patch/replay.
- Source/manifests search `plugin|Plugin|libloading` found no Pentomino loader/registration/unload implementation in crates. New lifecycle remains UNVERIFIED.

## Three strongest boundary risks

1. Existing core snapshots contain presentation and FlatLand assumptions. Existing core is not synonymous with proposed Tiny Core; preserve old serde structures via compatibility adapters.
2. World→Project→Scene.gameplay and view/render texture contracts propagate format/genre coupling even without a runtime normal renderer dependency. Do not require these as inputs to the new independent contract.
3. Data-only import, session-global ABI registry and filesystem-backed Project require explicit host trust and scoped lifecycle ownership. New plugins must not inherit global sessions or executable import authority. Unload/isolation safety UNVERIFIED.

## Test paths and tools

`docs/TESTPLAN.md`, `.github/workflows/ci.yml`: fmt/clippy/workspace tests/example replays/no-default-features headless. `.github/workflows/client.yml`: Flutter3.47.6 analyze/test, Windows release/embedded UTF-8 launch; Android Java21/three Rust targets/APK.

Relevant regression symbols: runtime tests `rc4.rs:47` view_is_simulation_invariant_with_motion_and_collision; :73 persistent_view_survives_goto_scoped_camera_save_and_scene_without_local_preset; :197 cosmetic_skip_does_not_repeat_damage_pp_or_rng; `rc5.rs:21` semantic_footprint_defaults_solid_and_view_cannot_grant_roof_access; :125 three_actor_bubbles_resume_once_and_camera_is_scoped_and_eased; client tests `final_resume.rs:19` only_content_revision_failure_exposes_typed_archive_reason.

PATH probe: python3/java found; cargo/rustc/rustup/flutter/dart/cmake/ninja/adb not found. Absolute-path installations UNVERIFIED. All engine/client tests NOT RUN — UNVERIFIED. Parent tooling results are logged separately.

## Known Windows failure

Handoff run37486665662/job112348341230 failed final_library_test digest comparison, later build/launch skipped. `client/test/final_library_test.dart:145,162` call fixture.package separately; :163 expects equal digest. `client/test/library_test.dart:48–51` builds new Archive/ZipEncoder bytes without explicitly fixed timestamps. `client/lib/game_library.dart:407` hashes entire ZIP bytes. Timestamp causality, reproduction and fix are UNVERIFIED. Preserve data retention assertions and digest/save validation.

## Inventory commands

Scout read git HEAD/status/remotes, Cargo/client manifests, relevant symbols and tests, source plugin search and PATH `command -v` probes. Parent independently confirmed HEAD/status/remotes and workspace manifest. Read-only review does not prove runtime gates G1–G7.
