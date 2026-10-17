pub mod settings;

use crate::integrations::{apps, power, sunshine};
use serde::{Deserialize, Serialize};
use settings::{Settings, SettingsStore};
use tauri::{Manager, State};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceId {
    LocalPc,
    GamerWake,
    SwitchStreaming,
    Ps5,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionId {
    Wake,
    Shutdown,
    OpenRemotePlay,
    OpenSunshine,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub action_id: ActionId,
    pub enabled: bool,
    pub reason: Option<String>,
    pub confirmation_required: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub status: String,
    pub detail: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub devices: Vec<Device>,
    pub checked_at: u128,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionRequest {
    pub device_id: DeviceId,
    pub action_id: ActionId,
    #[serde(default)]
    pub confirmed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    message: String,
}

fn capability(
    action_id: ActionId,
    enabled: bool,
    reason: &str,
    confirmation_required: bool,
) -> Capability {
    Capability {
        action_id,
        enabled,
        reason: (!enabled).then(|| reason.to_owned()),
        confirmation_required,
    }
}

fn validate_action(request: &ActionRequest) -> Result<(), String> {
    let allowed = matches!(
        (request.device_id, request.action_id),
        (DeviceId::LocalPc, ActionId::Shutdown)
            | (DeviceId::GamerWake, ActionId::Wake)
            | (DeviceId::SwitchStreaming, ActionId::OpenSunshine)
            | (DeviceId::Ps5, ActionId::OpenRemotePlay)
    );
    if !allowed {
        return Err("Esta ação não pertence ao dispositivo selecionado.".into());
    }
    if request.action_id == ActionId::Shutdown && !request.confirmed {
        return Err("Confirme o desligamento deste computador antes de continuar.".into());
    }
    Ok(())
}

fn read_catalog(settings: &Settings) -> Catalog {
    let remote_play = apps::remote_play_path().is_some();
    let sunshine = sunshine::probe(settings.sunshine_port);
    let machine_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Este computador".into());
    let wake_configured = settings.wake_target.is_some();
    let devices = vec![
        Device {
            id: DeviceId::LocalPc, name: "Este computador".into(), status: "Online".into(),
            detail: format!("{machine_name} · {}", std::env::consts::OS),
            capabilities: vec![capability(ActionId::Shutdown, cfg!(target_os = "windows"), "Disponível no Windows nesta etapa.", true)],
        },
        Device {
            id: DeviceId::GamerWake, name: "Acordar PC gamer".into(),
            status: if wake_configured { "Destino salvo" } else { "A configurar" }.into(),
            detail: "O despertar deve partir de outro aparelho ativo. Enviar o pacote não confirma que o destino ligou.".into(),
            capabilities: vec![capability(ActionId::Wake, wake_configured, "Salve o MAC e o broadcast do PC gamer primeiro.", false)],
        },
        Device {
            id: DeviceId::SwitchStreaming, name: "Streaming para Switch".into(),
            status: if sunshine.reachable { "Ligado" } else { "Indisponível" }.into(),

            detail: sunshine.detail,
            capabilities: vec![capability(ActionId::OpenSunshine, sunshine.installed || sunshine.reachable, "Sunshine não detectado neste PC.", false)],
        },
        Device {
            id: DeviceId::Ps5, name: "PlayStation 5".into(),
            status: if remote_play { "Remote Play instalado" } else { "App não encontrado" }.into(),
            detail: "A conexão e o estado do console são verificados no aplicativo da Sony.".into(),
            capabilities: vec![capability(ActionId::OpenRemotePlay, remote_play, "Instale o PS Remote Play neste computador.", false)],
        },
    ];
    Catalog {
        devices,
        checked_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
    }
}

pub fn setup_store(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let path = app.path().app_config_dir()?.join("controls.json");
    app.manage(SettingsStore::new(path));
    Ok(())
}

#[tauri::command]
pub fn control_settings(store: State<'_, SettingsStore>) -> Result<Settings, String> {
    store.load()
}

#[tauri::command]
pub fn save_control_settings(
    settings: Settings,
    store: State<'_, SettingsStore>,
) -> Result<(), String> {
    store.save(&settings)
}

#[tauri::command]
pub async fn device_catalog(store: State<'_, SettingsStore>) -> Result<Catalog, String> {
    let settings = store.load()?;
    tauri::async_runtime::spawn_blocking(move || read_catalog(&settings))
        .await
        .map_err(|_| "Não foi possível consultar os dispositivos locais.".into())
}

#[tauri::command]
pub async fn execute_action(
    app: tauri::AppHandle,
    store: State<'_, SettingsStore>,
    request: ActionRequest,
) -> Result<ActionResult, String> {
    validate_action(&request)?;
    let settings = store.load()?;
    let message = match (request.device_id, request.action_id) {
        (DeviceId::GamerWake, ActionId::Wake) => {
            let target = settings
                .wake_target
                .ok_or("Configure o destino Wake-on-LAN primeiro.")?;
            tauri::async_runtime::spawn_blocking(move || {
                power::wake(&target.mac, &target.broadcast)
            })
            .await
            .map_err(|_| "Não foi possível enviar o despertar.")??;
            "Pacote de despertar enviado. O estado do PC ainda não foi confirmado."
        }
        (DeviceId::LocalPc, ActionId::Shutdown) => {
            power::shutdown_local(true)?;
            "Desligamento solicitado ao Windows."
        }
        (DeviceId::Ps5, ActionId::OpenRemotePlay) => {
            apps::launch_remote_play()?;
            "PS Remote Play aberto. Conecte ao console pelo aplicativo."
        }
        (DeviceId::SwitchStreaming, ActionId::OpenSunshine) => {
            sunshine::open_panel(&app, settings.sunshine_port)?;
            "Painel Sunshine aberto no navegador. O pareamento é realizado por ele."
        }
        _ => return Err("Ação não suportada.".into()),
    };
    Ok(ActionResult {
        message: message.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_cannot_be_routed_to_the_wrong_device() {
        assert!(validate_action(&ActionRequest {
            device_id: DeviceId::GamerWake,
            action_id: ActionId::Shutdown,
            confirmed: true
        })
        .is_err());
        assert!(validate_action(&ActionRequest {
            device_id: DeviceId::Ps5,
            action_id: ActionId::Wake,
            confirmed: false
        })
        .is_err());
    }

    #[test]
    fn local_shutdown_requires_confirmation_before_dispatch() {
        assert!(validate_action(&ActionRequest {
            device_id: DeviceId::LocalPc,
            action_id: ActionId::Shutdown,
            confirmed: false
        })
        .is_err());
        assert!(validate_action(&ActionRequest {
            device_id: DeviceId::LocalPc,
            action_id: ActionId::Shutdown,
            confirmed: true
        })
        .is_ok());
    }

    #[test]
    fn arbitrary_shell_actions_are_rejected_during_deserialization() {
        assert!(serde_json::from_str::<ActionRequest>(
            r#"{"deviceId":"local_pc","actionId":"run_shell","confirmed":true}"#
        )
        .is_err());
    }
}
