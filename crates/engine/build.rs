fn main() {
    // Tell cargo to rerun this build script if shaders change
    println!("cargo:rerun-if-changed=src/layers/renderer/shader.wgsl");
    println!("cargo:rerun-if-changed=src/layers/raytracer/raytracer.wgsl");

    // Validate all shader files at compile time
    validate_shader("src/layers/renderer/shader.wgsl");
    validate_shader("src/layers/raytracer/raytracer.wgsl");
}

fn validate_shader(path: &str) {
    let shader_source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read shader file {}: {}", path, e));

    // Parse the WGSL shader
    let module = match naga::front::wgsl::parse_str(&shader_source) {
        Ok(module) => module,
        Err(e) => {
            panic!("Shader parsing failed for {}:\n{:?}", path, e);
        }
    };

    // Validate the parsed shader module
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    match validator.validate(&module) {
        Ok(_module_info) => {
            println!("cargo:warning=✓ Shader validated: {}", path);
        }
        Err(e) => {
            panic!("Shader validation failed for {}:\n{:?}", path, e);
        }
    }
}
