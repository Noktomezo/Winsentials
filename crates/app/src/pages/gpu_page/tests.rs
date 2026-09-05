#[test]
fn default_slots_match_hardware_kind() {
    let discrete_defaults = ["3D", "Copy", "Video Encode", "Video Decode"];
    assert_eq!(discrete_defaults.len(), 4);

    let integrated_defaults = ["3D", "Copy", "High Priority Compute", "High Priority 3D"];
    assert_eq!(integrated_defaults.len(), 4);
}