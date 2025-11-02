fn main() {
    // Auto-discover modules and inject prelude imports
    let config = build_utils::AutoModConfig::new("src")
        .with_prelude("trialogue_engine::prelude::*")
        .with_prelude("trialogue_editor::prelude::*");

    build_utils::auto_discover_modules(config).expect("Failed to auto-discover modules");

    // Tell cargo to rerun this build script if source files change
    println!("cargo:rerun-if-changed=src");
}
