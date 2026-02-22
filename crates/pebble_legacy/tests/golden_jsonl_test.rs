use pebble::store::JsonlStore;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tempfile::TempDir;

fn read_jsonl_values(path: &Path) -> Vec<serde_json::Value> {
    let file = File::open(path).expect("Failed to open JSONL file");
    let reader = BufReader::new(file);
    let deserializer = serde_json::Deserializer::from_reader(reader);
    deserializer
        .into_iter::<serde_json::Value>()
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse JSONL values")
}

#[test]
fn test_golden_jsonl_round_trip_preserves_fields() {
    let fixture_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden.jsonl");

    let original_values = read_jsonl_values(&fixture_path);
    let store = JsonlStore::new(
        fixture_path
            .to_str()
            .expect("Fixture path should be valid UTF-8"),
    );
    let issues = store.read_issues().expect("Failed to parse golden.jsonl");

    assert_eq!(
        issues.len(),
        original_values.len(),
        "Golden JSONL should round-trip the same number of issues"
    );

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("issues.jsonl");
    let output_store = JsonlStore::new(
        output_path
            .to_str()
            .expect("Output path should be valid UTF-8"),
    );
    output_store
        .write_issues(&issues)
        .expect("Failed to write round-trip issues");

    let round_trip_values = read_jsonl_values(&output_path);
    assert_eq!(
        original_values, round_trip_values,
        "Round-tripped JSONL should preserve all fields"
    );
}
