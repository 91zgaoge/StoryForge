//! Compilation-level contract test for the creative_engine <-> canonical_state
//! cycle break.

#[test]
fn asset_snapshot_does_not_import_canonical_state() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/creative_engine/asset_snapshot.rs");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(!content.contains("canonical_state"));
}
