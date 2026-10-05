//! Revision-checked, stable-ID resource patches validated before replacing authored bytes.
use crate::{Error, Project, Result, atomic_bytes, read_text};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
pub fn revision(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn resource<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Value> {
    let mut value = value;
    for id in path.split('/').filter(|s| !s.is_empty()) {
        value = match value {
            Value::Object(o) => o.get_mut(id),
            Value::Array(a) => a
                .iter_mut()
                .find(|v| v.get("id").and_then(Value::as_str) == Some(id)),
            _ => None,
        }
        .ok_or_else(|| Error(format!("patch: missing resource segment {id}")))?;
    }
    Ok(value)
}
pub fn inspect(project: &Project, file: &str, path: &str) -> Result<Value> {
    if !project.files_checked.iter().any(|p| p == file) || !file.ends_with(".json5") {
        return Err(Error("resource requires checked authored JSON5".into()));
    }
    let bytes = fs::read(project.path(file)?).map_err(|e| Error(format!("resource file: {e}")))?;
    let mut v: Value =
        json5::from_str(std::str::from_utf8(&bytes).map_err(|e| Error(e.to_string()))?)
            .map_err(|e| Error(e.to_string()))?;
    let part = resource(&mut v, path)?.clone();
    Ok(json!({"ok":true,"file":file,"resource":path,"revision":revision(&bytes),"value":part}))
}
pub fn apply(
    project: &Project,
    file: &str,
    path: &str,
    expected: &str,
    patch: Value,
) -> Result<Value> {
    if !project.files_checked.iter().any(|p| p == file)
        || !file.ends_with(".json5")
        || file == "ge4g.toml"
    {
        return Err(Error(
            "patch requires an authored JSON5 resource file".into(),
        ));
    }
    let destination = project.path(file)?;
    let before = fs::read(&destination).map_err(|e| Error(e.to_string()))?;
    if revision(&before) != expected {
        return Err(Error(
            "patch revision conflict; inspect the resource again".into(),
        ));
    }
    let mut doc: Value =
        json5::from_str(&read_text(&destination)?).map_err(|e| Error(e.to_string()))?;
    if !patch.is_object() {
        return Err(Error("resource patch must be an object".into()));
    }
    crate::flatland::merge(resource(&mut doc, path)?, patch);
    let bytes = serde_json::to_vec_pretty(&doc).map_err(|e| Error(e.to_string()))?;
    let staging = tempfile::tempdir().map_err(|e| Error(e.to_string()))?;
    fn copy(source: &Path, destination: &Path, count: &mut usize) -> Result<()> {
        fs::create_dir_all(destination).map_err(|e| Error(e.to_string()))?;
        for entry in fs::read_dir(source).map_err(|e| Error(e.to_string()))? {
            let e = entry.map_err(|e| Error(e.to_string()))?;
            let t = e.file_type().map_err(|e| Error(e.to_string()))?;
            if t.is_symlink() {
                return Err(Error("patch staging rejects symlinks".into()));
            }
            let name = e.file_name();
            if [".git", "target", "dist", "artifacts", "__pycache__"]
                .iter()
                .any(|x| name == *x)
            {
                continue;
            }
            *count += 1;
            if *count > 4096 {
                return Err(Error("patch staging file limit 4096".into()));
            }
            if t.is_dir() {
                copy(&e.path(), &destination.join(name), count)?;
            } else if t.is_file() {
                if e.metadata().map_err(|e| Error(e.to_string()))?.len() > 16 * 1024 * 1024 {
                    return Err(Error("patch staging file exceeds 16MiB".into()));
                }
                fs::copy(e.path(), destination.join(name)).map_err(|e| Error(e.to_string()))?;
            }
        }
        Ok(())
    }
    copy(&project.root, staging.path(), &mut 0)?;
    atomic_bytes(&staging.path().join(file), &bytes)?;
    Project::load(staging.path())?;
    // Recheck after validation to avoid replacing a file changed by another author.
    let current = fs::read(&destination).map_err(|e| Error(e.to_string()))?;
    if current != before {
        return Err(Error("patch revision changed during validation".into()));
    }
    atomic_bytes(&destination, &bytes)?;
    Ok(
        json!({"ok":true,"file":file,"resource":path,"revision":revision(&bytes),"bytes":bytes.len()}),
    )
}
