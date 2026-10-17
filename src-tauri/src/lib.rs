mod core;
mod desktop;
mod integrations;
#[cfg(feature = "lighting-tuya")]
mod tuya;
mod window;

pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Err(error) = desktop::show(app) {
                eprintln!("Falha ao recuperar a janela: {error}");
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
                .build(),
        )
        .setup(|app| {
            core::setup_store(app)?;
            desktop::setup(app)
        });

    #[cfg(not(feature = "lighting-tuya"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        window::resize_panel,
        core::device_catalog,
        core::control_settings,
        core::save_control_settings,
        core::execute_action,
        desktop::hide_widget,
        desktop::quit_app,
    ]);
    #[cfg(feature = "lighting-tuya")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        window::resize_panel,
        core::device_catalog,
        core::control_settings,
        core::save_control_settings,
        core::execute_action,
        desktop::hide_widget,
        desktop::quit_app,
        tuya::tuya_connection,
        tuya::open_tuya_guide,
        tuya::connect_tuya,
        tuya::tuya_status,
        tuya::set_tuya_power,
        tuya::disconnect_tuya,
    ]);

    builder
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                match window.hide() {
                    Ok(()) => api.prevent_close(),
                    Err(error) => eprintln!("Falha ao ocultar a janela: {error}"),
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Falha ao iniciar o Sync System");
}
