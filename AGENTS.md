# Agent Guidelines for Trialogue

## Build & Test Commands
- **Build**: `cargo build` or `cargo build --release`
- **Run**: `cargo run` (runs the game crate)
- **Lint**: `cargo fmt --all -- --check` and `cargo check`
- **Format**: `cargo fmt --all`
- **Test all**: `cargo test --workspace`
- **Test single**: `cargo test <test_name>` or `cargo test -p <crate_name> <test_name>`

## Code Style
- **Toolchain**: Rust nightly (see .github/workflows/ci.yaml)
- **Imports**: Use prelude modules (`use crate::prelude::*;`) - each crate has its own prelude that re-exports common types
- **Components**: Use `#[derive(Component)]` from bevy_ecs; components often have GPU variants (see gpu_component.rs)
- **Types**: Prefer nalgebra types (Point3, Vector3, UnitQuaternion, Matrix4) from prelude
- **Error handling**: Use `anyhow::Result<T>` as primary error type (see engine/src/lib.rs)
- **Naming**: snake_case for functions/modules, PascalCase for types, SCREAMING_SNAKE for constants
- **Layers**: Follow the Layer trait pattern for extensibility (see layers/ dirs)
- **Tests**: Use `#[cfg(test)]` modules with `#[test]` functions (see raycast.rs:89-122)

## Architecture Notes
- Workspace structure: engine (core), editor (UI), game (application), build-utils, auto-prelude
- ECS-based using bevy_ecs with custom Layer system for frame updates
- GPU components follow a pattern: user component + GPU variant + GpuComponent trait (see transform.rs)
