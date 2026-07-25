//! Compile-time guard: ensure write_time_bundle.rs no longer directly imports
//! story_system::StorySystemEngine after the RuntimeContractProvider port is
//! introduced.

#[test]
fn write_time_bundle_load_does_not_import_story_system() {
    let content = std::fs::read_to_string("src/creative_engine/write_time_bundle.rs").unwrap();
    assert!(!content.contains("story_system::StorySystemEngine"));
}
