//! Compilation-level contract test for the creative_engine <-> canonical_state
//! cycle break.

#[test]
fn asset_snapshot_does_not_import_canonical_state() {
    let content = std::fs::read_to_string("src/creative_engine/asset_snapshot.rs").unwrap();
    assert!(!content.contains("canonical_state"));
}
