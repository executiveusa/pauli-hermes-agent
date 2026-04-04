#![forbid(unsafe_code)]

use jsonschema::Validator;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures_dir() -> PathBuf {
    repo_root().join("tests/ext_conformance/fixtures")
}

fn schema_path() -> PathBuf {
    repo_root().join("docs/schema/mock_spec.json")
}

fn compiled_mock_spec_schema() -> Validator {
    let schema_path = schema_path();
    let raw = fs::read_to_string(&schema_path)
        .unwrap_or_else(|err| panic!("Failed to read schema {}: {err}", schema_path.display()));
    let schema: Value = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("Failed to parse schema {}: {err}", schema_path.display()));

    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap_or_else(|err| panic!("Failed to compile schema {}: {err}", schema_path.display()))
}

fn list_mock_spec_fixtures(dir: &Path) -> Vec<PathBuf> {
    let mut files = fs::read_dir(dir)
        .expect("read_dir fixtures")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "json")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("mock_spec_"))
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn mock_spec_fixtures_validate_against_schema() {
    let schema = compiled_mock_spec_schema();
    let dir = fixtures_dir();
    let files = list_mock_spec_fixtures(&dir);
    assert!(
        !files.is_empty(),
        "no mock spec fixtures found in {}",
        dir.display()
    );

    for path in files {
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("Failed to read fixture {}: {err}", path.display()));
        let instance: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("Failed to parse fixture JSON {}: {err}", path.display()));

        if let Err(err) = schema.validate(&instance) {
            panic!("Fixture {} does not validate: {err}", path.display());
        }
    }
}
