fn main() {
    let attributes =
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[
            "reset",
            "load_demo",
            "load_program",
            "run",
            "snapshot",
            "framebuffer",
            "read_memory",
        ]));

    #[cfg(windows)]
    let attributes =
        attributes.windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());

    tauri_build::try_build(attributes).expect("failed to run tauri-build");

    #[cfg(windows)]
    embed_manifest_everywhere();
}

#[cfg(windows)]
fn embed_manifest_everywhere() {
    use std::{env, fs, path::PathBuf};

    const MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency><dependentAssembly><assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0" processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" /></dependentAssembly></dependency>
</assembly>
"#;

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    let manifest = out.join("app.manifest");
    fs::write(&manifest, MANIFEST).expect("write the manifest");
    let rc = out.join("app-manifest.rc");
    let path = manifest.display().to_string().replace('\\', "\\\\");
    fs::write(&rc, format!("1 24 \"{path}\"\n")).expect("write the resource script");
    embed_resource::compile_for_everything(rc, embed_resource::NONE)
        .manifest_required()
        .expect("embed the manifest");
}
