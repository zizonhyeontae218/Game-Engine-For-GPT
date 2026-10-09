# P3 Camera scout — actual repository and fifth-fixture seams

Read-only child scout did not edit/build/test. Parent confirmed branch2159304,
main2d1ffd0 (P2 unmerged), PR3 all latest CI SUCCESS, external public Core23/23
ACCEPTED; no P2 blocking defect. P3 stacks on P2; external legacy remains UNVERIFIED.
No genuine Tiny Core API blocker found.

## P2 public consumer surface

- p2/types.rs:280-294 Selection/ReadFrame: owner/schema filters; tick, owned
  records/scenes/entities/history. Objects contain identity/scene scope, no position.
- p2/host.rs:365-413 select takes &self and clones selected data. Multiple views
  should share one ReadFrame when possible; repeated selection cost must be measured.
- p2/types.rs:304-311 CoreDiscovery exposes declared schemas/capabilities/limits.
  Camera-specific discovery belongs to a separate external presentation layer.
- Declarative typed I64 records can encode positions/units; View interprets them.
  No Core float/position/camera additions necessary. A separate presentation host
  passes only &ReadFrame to linked format consumers, not Core transactions.
- ReadFrame structured RecordKey map is a Rust DTO, not a JSON frame/save wire.
  New render frame/local presentation save need their own declared canonical wire.

## Legacy classification

| Classification | Actual seam / reason |
|---|---|
| Reusable concept | World snapshot/render_snapshot readonly (runtime/lib.rs258-263); public CPU ge4g_render2d::render(Project,Snapshot,bool), Frame dimensions/rgba/write_png |
| Reusable concept | Follow+clamp(runtime/gameplay.rs1276-1290), ground projection versus upright feet(render2d/lib.rs223-242, projection.rs69-89), tick smoothstep(runtime/view.rs36-49) |
| Compatibility-only | Snapshot.camera singleton(core/lib.rs229-243), target+bounds(project/gameplay.rs167-170), percent zoom/tilt/shear View(:55-63), saved legacy Systems presentation |
| Native replacement | Multi CameraId/viewports/routing, explicit spaces/projections, composable deterministic behaviors/transitions, camera-local lifecycle/save/discovery |
| Obsolete for native use | Hidden singleton/preset/system-JSON ownership and fixed12tick feature-gated position-only blend as the generic camera contract |

Legacy Projection module is private(render2d/lib.rs8). Native math cannot import it.
Adapter may clone immutable Project/Snapshot, apply presentation-only overrides
from actual native frame and call public render; actual native-projection influence
and pixels must be asserted. Never warp a completed framebuffer: ground geometry
and upright feet must retain separate projection semantics.

## Verified fifth legacy demo

`examples/flatland_nuvema`, Nuvema Field Study/마름꽃마을, introduced61eca95 rc3
(history doc development-history-through-rc4.md9-10). Five scenes:
town/home/friend_home/friend_home_right/lab;320x240, start town.69PNG and own
CC0 license/audio reuse; no Pokemon original graphics/music. No asset-licenses.json.
Keep as existing acceptance fixture, not a newly promoted public demo.

Town presets(town.json5:2909-2929): plan100/100/0, depth115/75/12,
balcony125/70/18 zoom/tilt/shear. Player follow bounds[0,0,192,336]. Existing
journey1957ticks and expectedRGBA9a57d2fad6c3a96ecf1e4f1378b751729325b1907f5c89319ada082f4df361c3.

IMPORTANT: existing view-menu actions(town.json5:2774-2844) ALSO modify plane,
elevation and quest/state; balcony setsplane1/z24. Pure Camera invariance tests
must keep the same immutable Core/World data and change presentation locally,
not execute these menu gameplay actions. Keep legacy journey as separate regression.
Legacy Action::View is instant(runtime/gameplay.rs392-402). Position-only12tick
smoothstep needs cutscene_bubbles feature(runtime/view.rs44-69); this fixture does
not enable it. New native smooth pose/orientation/zoom/projection/view transitions
are new architecture, not a claim that old smooth behavior already exists here.

## Compatibility and acceptance

P2 legacy projection exports only id/state(pentomino-legacy/lib.rs107-152,205-228,
334-353). A NEW outside-Core adapter must map immutable World/Snapshot positions,
assets/ground/billboard metadata and installed Core stable refs. Current renderer/
legacy save/schema/ABI remain unchanged. Legacy camera is reference behavior only.
Test actual fifth entities/record imports, render-facing anchors/pixels using native
Camera output, Core byte equality/RNG/identity preservation, smooth mid-transition
save/replay and remove/reinstall; a direct old-render call alone is insufficient.
Native perspective/mixed representation support and actual model rasterization/full
split-screen UI are distinct scopes. Full renderer rewrite is excluded.

## Parent baseline execution

`PKG_CONFIG_PATH=<local ALSA SDK> cargo run --locked -p ge4g-cli -- test
examples/flatland_nuvema --json`: PASS,1957ticks, assertions1, deterministic,
save_reload,golden_checked,png_capture alltrue. Source base2159304; original log
/workspace/scratch/pentomino-p3-fifth-baseline.log. All P3 implementation/tests
UNVERIFIED at scout stage.
