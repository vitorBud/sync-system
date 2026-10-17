use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SunshineStatus {
    pub installed: bool,
    pub reachable: bool,
    pub detail: String,
}

#[derive(Deserialize)]
struct ServerInfo {
    #[serde(rename = "@status_code")]
    status_code: Option<u16>,
    state: Option<String>,
}

fn parse_server_info(text: &str) -> Option<bool> {
    let info: ServerInfo = quick_xml::de::from_str(text).ok()?;
    if info.status_code != Some(200) {
        return None;
    }
    match info.state.as_deref()? {
        "SUNSHINE_SERVER_FREE" => Some(false),
        "SUNSHINE_SERVER_BUSY" => Some(true),
        _ => None,
    }
}

pub fn probe(port: u16) -> SunshineStatus {
    let installed = ["ProgramFiles", "ProgramFiles(x86)"]
        .iter()
        .filter_map(std::env::var_os)
        .any(|base| {
            PathBuf::from(base)
                .join("Sunshine")
                .join("sunshine.exe")
                .is_file()
        });
    let session = (|| {
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .timeout(Duration::from_millis(1000))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .ok()?;
        let response = client
            .get(format!("http://127.0.0.1:{port}/serverinfo"))
            .send()
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        parse_server_info(&response.text().ok()?)
    })();
    SunshineStatus {
        installed,
        reachable: session.is_some(),
        detail: if session.is_some() {
            "Sunshine está ligado e respondendo neste PC.".into()
        } else {
            "Sem resposta válida do Sunshine. Confira o serviço e a porta configurada.".into()
        },
    }
}

pub fn open_panel(app: &tauri::AppHandle, port: u16) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let admin_port = port
        .checked_add(1)
        .ok_or("Porta de administração inválida.")?;
    app.opener()
        .open_url(format!("https://localhost:{admin_port}"), None::<&str>)
        .map_err(|_| "Não foi possível abrir o painel Sunshine no navegador.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_free_and_busy_from_valid_sunshine_response() {
        assert_eq!(
            parse_server_info(
                r#"<root status_code="200"><state>SUNSHINE_SERVER_FREE</state></root>"#
            ),
            Some(false)
        );
        assert_eq!(
            parse_server_info(
                r#"<root status_code="200"><state>SUNSHINE_SERVER_BUSY</state></root>"#
            ),
            Some(true)
        );
    }

    #[test]
    fn unrelated_service_and_errors_are_not_reported_online() {
        for response in [
            "hello",
            "<html>Login</html>",
            r#"<root status_code="401"><state>SUNSHINE_SERVER_FREE</state></root>"#,
            r#"<root status_code="200"><state>UNKNOWN</state></root>"#,
        ] {
            assert_eq!(parse_server_info(response), None);
        }
    }
}
