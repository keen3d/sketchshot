use tauri::ipc::{InvokeBody, Request, Response};
use tauri::{Emitter, Manager};

/// 실행 인자에서 열 파일 경로를 찾음 (.sketchshot 더블클릭 등)
fn file_arg(args: &[String]) -> Option<String> {
    args.iter().skip(1).find(|a| !a.starts_with('-')).cloned()
}

/// 앱이 만든 데이터(작업 파일·PSD·이미지)를 사용자가 고른 경로에 씀.
/// 큰 파일도 빠르게 넘기도록 경로는 헤더로, 내용은 원시 바이트로 받음.
#[tauri::command]
fn write_file(request: Request) -> Result<(), String> {
    let raw = request
        .headers()
        .get("x-path")
        .ok_or("저장 경로가 없습니다")?
        .to_str()
        .map_err(|e| e.to_string())?;
    let path = urlencoding::decode(raw).map_err(|e| e.to_string())?.into_owned();
    match request.body() {
        InvokeBody::Raw(bytes) => std::fs::write(&path, bytes).map_err(|e| e.to_string()),
        _ => Err("저장할 데이터 형식이 올바르지 않습니다".into()),
    }
}

/// 사용자가 연 파일을 원시 바이트로 돌려줌
#[tauri::command]
fn read_file(path: String) -> Result<Response, String> {
    std::fs::read(&path).map(Response::new).map_err(|e| e.to_string())
}

/// 처음 실행될 때 함께 넘어온 파일 경로
#[tauri::command]
fn startup_file() -> Option<String> {
    file_arg(&std::env::args().collect::<Vec<_>>())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 이미 실행 중일 때 파일을 더블클릭하면 새 창 대신 기존 창으로 넘김
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(p) = file_arg(&argv) {
                let _ = app.emit("open-file", p);
            }
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![write_file, read_file, startup_file])
        .run(tauri::generate_context!())
        .expect("SketchShot을 실행하지 못했습니다");
}
