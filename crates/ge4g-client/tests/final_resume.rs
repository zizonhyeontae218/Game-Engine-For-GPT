use ge4g_client::request_json;
use serde_json::{Value, json};
use std::{fs, path::Path};
fn request(value: Value) -> Value {
    request_json(&value.to_string())
}
fn copy(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &target.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}
#[test]
fn only_content_revision_failure_exposes_typed_archive_reason() {
    let dir = tempfile::tempdir().unwrap();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_harbor"),
        dir.path(),
    );
    let project = dir.path().join("ge4g.toml");
    let save = dir.path().join("resume.json");
    let opened = request(json!({"op":"open","project":project}));
    let id = opened["session"].as_u64().unwrap();
    assert_eq!(
        request(json!({"op":"save","session":id,"path":save}))["ok"],
        true
    );
    request(json!({"op":"close","session":id}));
    let compatible = request(json!({"op":"open","project":project,"load":save}));
    assert_eq!(compatible["ok"], true);
    request(json!({"op":"close","session":compatible["session"]}));
    let manifest = fs::read_to_string(&project).unwrap();
    fs::write(&project, manifest.replacen("name=\"", "name=\"updated ", 1)).unwrap();
    let stale = request(json!({"op":"open","project":project,"load":save}));
    assert_eq!(stale["ok"], false);
    assert_eq!(stale["error_code"], "save_content_revision_mismatch");
    let fresh = request(json!({"op":"open","project":project}));
    assert_eq!(fresh["ok"], true);
    request(json!({"op":"close","session":fresh["session"]}));
    fs::write(&save, "{broken").unwrap();
    let corrupt = request(json!({"op":"open","project":project,"load":save}));
    assert_eq!(corrupt["ok"], false);
    assert!(corrupt.get("error_code").is_none());
}
