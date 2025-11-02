use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::Item;

pub struct AutoModConfig {
    /// Root directory to start scanning (typically "src")
    pub root_dir: PathBuf,
    /// Patterns to ignore (e.g., "build.rs", "main.rs")
    pub ignore_patterns: Vec<String>,
    /// Prelude imports to inject into files (e.g., vec!["crate::prelude::*"])
    pub prelude_imports: Vec<String>,
}

impl Default for AutoModConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("src"),
            ignore_patterns: vec![
                "mod.rs".to_string(),
                "lib.rs".to_string(),
                "main.rs".to_string(),
                "build.rs".to_string(),
            ],
            prelude_imports: vec![],
        }
    }
}

impl AutoModConfig {
    pub fn new<P: Into<PathBuf>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.into(),
            ..Default::default()
        }
    }

    pub fn ignore_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.ignore_patterns.push(pattern.into());
        self
    }

    pub fn with_prelude(mut self, import: impl Into<String>) -> Self {
        self.prelude_imports.push(import.into());
        self
    }
}

/// Auto-discover modules and generate/update mod.rs files
pub fn auto_discover_modules(config: AutoModConfig) -> std::io::Result<()> {
    process_directory(&config.root_dir, &config)?;
    Ok(())
}

/// Auto-discover modules with default configuration
pub fn auto_discover_modules_default() -> std::io::Result<()> {
    auto_discover_modules(AutoModConfig::default())
}

fn process_directory(dir: &Path, config: &AutoModConfig) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    // Find all .rs files in this directory (excluding ignored patterns)
    let mut module_names = Vec::new();
    let mut rs_files = Vec::new();
    let entries = fs::read_dir(dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            process_directory(&path, config)?;
        } else if let Some(extension) = path.extension() {
            if extension == "rs" {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip ignored patterns
                    if !config.ignore_patterns.iter().any(|p| file_name == p) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            module_names.push(stem.to_string());
                        }
                        rs_files.push(path.clone());
                    }
                }
            }
        }
    }

    // If there are modules to declare, ensure mod.rs exists and is up to date
    if !module_names.is_empty() {
        module_names.sort();
        update_or_create_mod_file(dir, &module_names, config)?;
    }

    // Inject prelude imports into all .rs files if configured
    if !config.prelude_imports.is_empty() {
        for file_path in rs_files {
            inject_prelude_imports(&file_path, config)?;
        }
    }

    Ok(())
}

fn update_or_create_mod_file(
    dir: &Path,
    module_names: &[String],
    config: &AutoModConfig,
) -> std::io::Result<()> {
    let mod_file_path = dir.join("mod.rs");

    if mod_file_path.exists() {
        // Parse existing mod.rs and add missing declarations
        update_existing_mod_file(&mod_file_path, module_names, config)?;
    } else {
        // Create new mod.rs
        create_new_mod_file(&mod_file_path, module_names, config)?;
    }

    Ok(())
}

fn update_existing_mod_file(
    mod_file_path: &Path,
    module_names: &[String],
    config: &AutoModConfig,
) -> std::io::Result<()> {
    let content = fs::read_to_string(mod_file_path)?;

    // Parse the file to find existing mod declarations
    let existing_mods = parse_existing_mods(&content);

    // Find missing modules
    let missing_mods: Vec<_> = module_names
        .iter()
        .filter(|name| !existing_mods.contains(name.as_str()))
        .collect();

    if missing_mods.is_empty() {
        return Ok(()); // Nothing to add
    }

    // Find the position to insert: after the last mod declaration, or at the beginning
    let lines: Vec<&str> = content.lines().collect();
    let mut result = String::new();
    let mut last_mod_line = None;

    // Find the last mod declaration
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("mod ") && line.trim().ends_with(';') {
            last_mod_line = Some(i);
        }
    }

    if let Some(last_idx) = last_mod_line {
        // Insert after the last mod declaration
        for (i, line) in lines.iter().enumerate() {
            result.push_str(line);
            result.push('\n');

            if i == last_idx {
                // Add new mod statements here
                for mod_name in &missing_mods {
                    result.push_str(&format!("mod {};\n", mod_name));
                }
            }
        }
    } else {
        // No existing mod declarations, add at the beginning
        for mod_name in &missing_mods {
            result.push_str(&format!("mod {};\n", mod_name));
        }
        result.push('\n');
        result.push_str(&content);
    }

    fs::write(mod_file_path, result)?;
    println!("Updated: {}", mod_file_path.display());

    Ok(())
}

fn create_new_mod_file(
    mod_file_path: &Path,
    module_names: &[String],
    _config: &AutoModConfig,
) -> std::io::Result<()> {
    let mut content = String::new();

    for mod_name in module_names {
        content.push_str(&format!("mod {};\n", mod_name));
    }

    fs::write(mod_file_path, content)?;
    println!("Created: {}", mod_file_path.display());

    Ok(())
}

fn parse_existing_mods(content: &str) -> HashSet<String> {
    let mut mods = HashSet::new();

    // Try to parse as Rust file
    if let Ok(file) = syn::parse_file(content) {
        for item in file.items {
            if let Item::Mod(item_mod) = item {
                mods.insert(item_mod.ident.to_string());
            }
        }
    }

    mods
}

fn inject_prelude_imports(file_path: &Path, config: &AutoModConfig) -> std::io::Result<()> {
    let content = fs::read_to_string(file_path)?;

    // Check which imports are missing
    let missing_imports: Vec<_> = config
        .prelude_imports
        .iter()
        .filter(|import| !content.contains(&format!("use {};", import)))
        .collect();

    if missing_imports.is_empty() {
        return Ok(()); // All imports already present
    }

    // Add missing imports at the beginning
    let mut prelude_section = String::new();
    for import in &missing_imports {
        prelude_section.push_str(&format!("use {};\n", import));
    }
    prelude_section.push('\n');

    let updated_content = format!("{}{}", prelude_section, content);
    fs::write(file_path, updated_content)?;
    println!("Injected prelude imports into: {}", file_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_existing_mods() {
        let content = r#"
            mod foo;
            mod bar;

            pub fn something() {
                // user code
            }

            mod baz;
        "#;

        let mods = parse_existing_mods(content);
        assert_eq!(mods.len(), 3);
        assert!(mods.contains("foo"));
        assert!(mods.contains("bar"));
        assert!(mods.contains("baz"));
    }
}
