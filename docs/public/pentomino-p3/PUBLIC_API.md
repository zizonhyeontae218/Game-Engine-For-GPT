# Public surface

Use ge4g_pentomino::p2::{CoreHost, CorePlugin, Selection, ReadFrame} for source
state, and ge4g_pentomino_view for presentation. All public signatures/types,
math conventions, bounds, errors, save rules and compatibility restrictions
are specified in CONTRACT.md. P2 types are documented in CORE_CONTRACT.md.

Native entry points: ViewHost::{new,install,replace,remove,add_camera,
remove_camera,activate,deactivate,set_active,update,frame,describe,save,hash,
restore}. Formats: Classic2DFormat, TopDownFormat, SideFormat, VerticalFormat.
Math: world_to_view/view_to_world/view_to_camera/camera_to_view/project/unproject.
CameraTarget holds independent pose, ViewTransform and Projection; ScreenPoint
includes normalized depth for invertible unprojection.

Compatibility crate: presentation_plugin(&World, owner) initializes a bounded
source snapshot with real P2 allocations; legacy_camera_target constructs a
supported presentation target; render_legacy consumes a CameraFrame and immutable
Project/Snapshot; legacy_foot anchors upright sprites on projected ground.

Representation::Model/Background/Billboard preserve 3D points and descriptors;
these APIs do not claim a complete mixed-scene renderer. Same-toolchain/target
floating-point determinism is supported; cross-platform bit equality is UNVERIFIED.

Compatibility source loading uses ge4g_project::Project::load(&std::path::Path)
and ge4g_runtime::World::new(Project). World exposes an immutable snapshot()
result (ge4g_core::Snapshot) and public project for this consumer. Use
examples/fifth_demo.rs to see actual source initialization and render calls.
The source importer snapshots the CURRENT legacy Scene; repeating imports
allocates new identities and is not continuous legacy gameplay hosting.

CameraStatus.current is sampled base pose; CameraFrame.target is the final
follow/shake/bounds result. SHA256-keyed SplitMix64 shake details, precision
limits, rendering subset and Camera-local save invariants are in CONTRACT.md.
