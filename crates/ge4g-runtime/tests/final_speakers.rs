use ge4g_project::{Project, gameplay::Instruction};
use ge4g_runtime::World;
use serde_json::{Value, json};
use std::path::Path;

fn bubble(speaker: Option<&str>, metadata: Value) -> World {
    let mut project = Project::load(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_harbor"),
    )
    .unwrap();
    let scene = project.scenes.get_mut("항구").unwrap();
    scene
        .entities
        .iter_mut()
        .find(|e| e.id == "minimal_actor")
        .unwrap()
        .metadata = serde_json::from_value(metadata).unwrap();
    scene.gameplay.events.insert(
        "speaker_test".into(),
        vec![
            Instruction::SayBubble {
                actor: "minimal_actor".into(),
                speaker: speaker.map(str::to_owned),
                text: "첫 줄".into(),
            },
            Instruction::SayBubble {
                actor: "minimal_actor".into(),
                speaker: speaker.map(str::to_owned),
                text: "둘째 줄".into(),
            },
            Instruction::Return { retain_view: false },
        ],
    );
    let mut world = World::new(project).unwrap();
    world.choose("continue").unwrap();
    world
        .command(
            &serde_json::from_value::<Vec<ge4g_project::flatland::Action>>(
                json!([{"op":"event_scene","event":"speaker_test"}]),
            )
            .unwrap(),
        )
        .unwrap();
    world
}
#[test]
fn bubble_speaker_is_optional_resolved_metadata_and_resume_exact() {
    for (authored, metadata, expected) in [
        (
            Some("안내인"),
            json!({"display_name":"다른 이름","name":"이름"}),
            Some("안내인"),
        ),
        (
            None,
            json!({"display_name":"선생님","name":"이름"}),
            Some("선생님"),
        ),
        (
            None,
            json!({"display_name":42,"name":"공방장"}),
            Some("공방장"),
        ),
        (Some(" "), json!({"display_name":"안내인"}), Some("안내인")),
        (None, json!({}), None),
    ] {
        let mut world = bubble(authored, metadata);
        let waiting = world.waiting().unwrap();
        assert_eq!(waiting["speaker"].as_str(), expected);
        assert_eq!(waiting["actor"], "minimal_actor");
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bubble.json");
        world.save(&path).unwrap();
        let mut resumed = World::with_save(world.project.clone(), Some(&path)).unwrap();
        assert_eq!(resumed.waiting(), world.waiting());
        resumed.choose("continue").unwrap();
        assert_eq!(resumed.waiting().unwrap()["text"], "둘째 줄");
        assert_eq!(resumed.waiting().unwrap()["speaker"].as_str(), expected);
        resumed.choose("continue").unwrap();
        assert!(resumed.waiting().is_none());
    }
}
