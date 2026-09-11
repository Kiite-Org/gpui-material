//! Parity test ensuring web/spec.js is in sync with Rust code.

use std::fs;
use std::path::Path;

use gpui_m3e::{SliderMetrics, SpringParams, SwitchMetrics, dark_scheme, light_scheme};

#[test]
fn test_web_spec_is_synced() {
    let spec_path = Path::new("web/spec.js");
    assert!(
        spec_path.exists(),
        "web/spec.js does not exist! Run 'cargo run --example export_spec' to generate it."
    );

    let content = fs::read_to_string(spec_path).expect("failed to read web/spec.js");

    // Check spring stiffness and damping
    let expr = SpringParams::expressive();
    let std_spring = SpringParams::standard();
    assert!(
        content.contains(&format!("stiffness: {:.1}", expr.stiffness)),
        "Expressive slider stiffness out of sync in web/spec.js!"
    );
    assert!(
        content.contains(&format!("damping: {:.1}", expr.damping)),
        "Expressive slider damping out of sync in web/spec.js!"
    );
    assert!(
        content.contains(&format!("stiffness: {:.1}", std_spring.stiffness)),
        "Standard switch stiffness out of sync in web/spec.js!"
    );

    // Check slider & switch metrics
    let sm = SliderMetrics::default();
    let swm = SwitchMetrics::default();
    assert!(
        content.contains(&format!("trackHeight: {:.1}", sm.track_height)),
        "Slider track height out of sync in web/spec.js!"
    );
    assert!(
        content.contains(&format!("trackWidth: {:.1}", swm.track_width)),
        "Switch track width out of sync in web/spec.js!"
    );
    assert!(
        content.contains(&format!("trackHeight: {:.1}", swm.track_height)),
        "Switch track height out of sync in web/spec.js!"
    );

    // Check dark theme primary
    let dark = dark_scheme();
    let dark_primary_hex = format!(
        "#{:02x}{:02x}{:02x}",
        (dark.primary.r * 255.0).round() as u8,
        (dark.primary.g * 255.0).round() as u8,
        (dark.primary.b * 255.0).round() as u8,
    );
    assert!(
        content.contains(&dark_primary_hex),
        "Dark theme primary out of sync in web/spec.js!"
    );

    // Check light theme primary
    let light = light_scheme();
    let light_primary_hex = format!(
        "#{:02x}{:02x}{:02x}",
        (light.primary.r * 255.0).round() as u8,
        (light.primary.g * 255.0).round() as u8,
        (light.primary.b * 255.0).round() as u8,
    );
    assert!(
        content.contains(&light_primary_hex),
        "Light theme primary out of sync in web/spec.js!"
    );
}
