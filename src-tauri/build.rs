fn main() {
    let attributes = if windows_msvc() {
        embed_manifest();
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())
    } else {
        tauri_build::Attributes::new()
    };
    tauri_build::try_build(attributes).expect("error while running tauri-build");
}

fn windows_msvc() -> bool {
    std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|env| env == "msvc")
}

/// Embed the Common Controls v6 manifest in every Windows executable.
///
/// `tauri_build` embeds one in the application only, as a resource. A test
/// executable that links the app crate has none, so Windows loads the old comctl32
/// and the program dies at startup with `STATUS_ENTRYPOINT_NOT_FOUND` before it
/// runs a test. Cargo has no link-arg setting that reaches the library's unit tests
/// and not the application, so the linker embeds the manifest in all of them and
/// `tauri_build` is told to leave its own out: two manifests are a duplicate-resource
/// link error. `windows-test.manifest` is the manifest `tauri_build` embeds by default.
fn embed_manifest() {
    let manifest = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("windows-test.manifest");
    println!("cargo:rerun-if-changed={}", manifest.display());
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
}
