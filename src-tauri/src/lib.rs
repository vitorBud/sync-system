#[tauri::command]
fn resize_panel(window: tauri::WebviewWindow, expanded: bool, height: Option<f64>) -> Result<(), String> {
    let height = if expanded { height.unwrap_or(580.0) } else { 80.0 };
    if !height.is_finite() || !(80.0..=1600.0).contains(&height) {
        return Err("Altura de janela inválida".into());
    }
    window.set_size(tauri::LogicalSize::new(384.0, height)).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![resize_panel])
        .run(tauri::generate_context!())
        .expect("Falha ao iniciar o Sync System");
}