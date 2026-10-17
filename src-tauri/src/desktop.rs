use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn show(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Mostrar Sync System", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Ocultar widget", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;
    let icon = app
        .default_window_icon()
        .ok_or("Ícone do aplicativo indisponível")?
        .clone();

    TrayIconBuilder::with_id("sync-system")
        .icon(icon)
        .tooltip("Sync System")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let result = match event.id.as_ref() {
                "show" => show(app),
                "hide" => app.get_webview_window("main").map_or(Ok(()), |w| w.hide()),
                "quit" => {
                    app.exit(0);
                    Ok(())
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                eprintln!("Falha na ação da bandeja: {error}");
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Err(error) = show(tray.app_handle()) {
                    eprintln!("Falha ao mostrar o widget: {error}");
                }
            }
        })
        .build(app)?;

    // Start visible even if the app was hidden when the position was saved.
    show(app.handle())?;
    Ok(())
}

#[tauri::command]
pub fn hide_widget(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
