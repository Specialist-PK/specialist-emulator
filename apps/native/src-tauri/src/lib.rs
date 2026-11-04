mod commands;

pub use commands::{AppState, CommandError, EmulatorSnapshot};

use tauri_specta::{Builder, collect_commands};

pub fn specta_builder<R: tauri::Runtime>() -> Builder<R> {
    Builder::<R>::new().commands(collect_commands![
        commands::reset,
        commands::load_demo,
        commands::load_monitor2_rom,
        commands::load_program,
        commands::run,
        commands::snapshot,
        commands::framebuffer,
        commands::read_memory
    ])
}

fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder::<tauri::Wry>();
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);
            Ok(())
        })
        .run(context())
        .expect("error while running Specialist emulator");
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;
    use specta_typescript::Typescript;
    use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
    use tauri::webview::InvokeRequest;
    use tauri::{
        WebviewWindowBuilder,
        ipc::{CallbackFn, InvokeBody},
    };

    use super::*;

    #[test]
    fn bindings_are_up_to_date() {
        let committed = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
        let fresh =
            std::env::temp_dir().join(format!("specialist-bindings-{}.ts", std::process::id()));
        specta_builder::<tauri::Wry>()
            .export(Typescript::default(), &fresh)
            .expect("failed to export bindings");
        let fresh_text = std::fs::read_to_string(&fresh).unwrap();
        std::fs::remove_file(&fresh).ok();

        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            std::fs::write(&committed, &fresh_text).unwrap();
            return;
        }
        let committed_text = std::fs::read_to_string(&committed).unwrap_or_default();
        assert!(
            committed_text == fresh_text,
            "apps/native/src/bindings.ts is stale; run `bun run update:bindings`"
        );
    }

    fn invoke(
        webview: &tauri::WebviewWindow<MockRuntime>,
        command: &str,
        args: serde_json::Value,
    ) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
        match get_ipc_response(
            webview,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: if cfg!(any(windows, target_os = "android")) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        ) {
            Ok(body) => Ok(body),
            Err(body) => Err(body),
        }
    }

    #[test]
    fn demo_runs_over_ipc_and_exposes_the_framebuffer() {
        let app = mock_builder()
            .manage(AppState::default())
            .invoke_handler(specta_builder::<MockRuntime>().invoke_handler())
            .build(context())
            .expect("failed to build app");
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let loaded = invoke(&webview, "load_demo", json!({}))
            .expect("load_demo failed")
            .deserialize::<serde_json::Value>()
            .unwrap();
        assert_eq!(loaded["cpu"]["pc"], 0);

        let stopped = invoke(&webview, "run", json!({ "instructions": 200_000 }))
            .expect("run failed")
            .deserialize::<serde_json::Value>()
            .unwrap();
        assert_eq!(stopped["cpu"]["pc"], 0x14);
        assert_eq!(stopped["cpu"]["halted"], true);

        let frame = invoke(&webview, "framebuffer", json!({}))
            .expect("framebuffer failed")
            .deserialize::<Vec<u8>>()
            .unwrap();
        assert_eq!(frame.len(), 12_288);
        assert_eq!(frame[1], 0x91);
        assert_eq!(frame[12_287], 0x40);
    }

    #[test]
    fn monitor2_rom_loads_and_runs_over_ipc() {
        let app = mock_builder()
            .manage(AppState::default())
            .invoke_handler(specta_builder::<MockRuntime>().invoke_handler())
            .build(context())
            .expect("failed to build app");
        let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();

        let error = invoke(&webview, "load_monitor2_rom", json!({ "rom": [0] }))
            .expect_err("a short ROM must be rejected");
        assert_eq!(error["kind"], "invalidRomSize");

        let rom = vec![0; 0x1800];
        let loaded = invoke(&webview, "load_monitor2_rom", json!({ "rom": rom }))
            .expect("load_monitor2_rom failed")
            .deserialize::<serde_json::Value>()
            .unwrap();
        assert_eq!(loaded["cpu"]["pc"], 0xc000);

        let stopped = invoke(&webview, "run", json!({ "instructions": 1 }))
            .expect("run failed")
            .deserialize::<serde_json::Value>()
            .unwrap();
        assert_eq!(stopped["cpu"]["pc"], 0xc001);
    }
}
