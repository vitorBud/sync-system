use hmac::{Hmac, Mac};
use reqwest::{blocking::Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SERVICE: &str = "com.sync.desktop.tuya";
const ACCOUNT: &str = "room-light";

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    region: String,
    client_id: String,
    client_secret: String,
    device_id: String,
    #[serde(default)]
    relay_code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightStatus {
    configured: bool,
    online: Option<bool>,
    on: Option<bool>,
    device_name: Option<String>,
    relay_code: Option<String>,
}

impl LightStatus {
    fn disconnected() -> Self {
        Self {
            configured: false,
            online: None,
            on: None,
            device_name: None,
            relay_code: None,
        }
    }
}

fn endpoint(region: &str) -> Result<&'static str, String> {
    match region {
        "us" => Ok("https://openapi.tuyaus.com"),
        "us-east" => Ok("https://openapi-ueaz.tuyaus.com"),
        "eu" => Ok("https://openapi.tuyaeu.com"),
        "eu-west" => Ok("https://openapi-weaz.tuyaeu.com"),
        "cn" => Ok("https://openapi.tuyacn.com"),
        "in" => Ok("https://openapi.tuyain.com"),
        _ => Err("Selecione o data center do seu projeto Tuya.".into()),
    }
}

fn identifier(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

impl Config {
    fn validate(&self) -> Result<(), String> {
        endpoint(&self.region)?;
        if !identifier(&self.client_id, 96) || !identifier(&self.device_id, 96) {
            return Err("Confira o Client ID e o Device ID no projeto Tuya.".into());
        }
        if !(8..=128).contains(&self.client_secret.len())
            || !self.client_secret.bytes().all(|c| c.is_ascii_graphic())
        {
            return Err("Confira o Client Secret do projeto Tuya.".into());
        }
        if !self.relay_code.is_empty()
            && (!identifier(&self.relay_code, 64) || !self.relay_code.starts_with("switch"))
        {
            return Err("O canal precisa ser um interruptor da tomada.".into());
        }
        Ok(())
    }
}

fn credential() -> Result<keyring::Entry, String> {
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return Err("Armazenamento protegido ainda não disponível neste sistema.".into());
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|_| "Não foi possível acessar o cofre de credenciais do sistema.".into())
}

fn load_config() -> Result<Option<Config>, String> {
    match credential()?.get_password() {
        Ok(value) => {
            let config: Config = serde_json::from_str(&value)
                .map_err(|_| "Configuração Tuya inválida. Conecte a tomada novamente.")?;
            config.validate()?;
            Ok(Some(config))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Não foi possível ler a conexão Tuya no cofre do sistema.".into()),
    }
}

fn store_config(config: &Config) -> Result<(), String> {
    let value =
        serde_json::to_string(config).map_err(|_| "Não foi possível preparar a conexão.")?;
    credential()?
        .set_password(&value)
        .map_err(|_| "Não foi possível salvar a conexão no cofre do sistema.".into())
}

fn signature(
    config: &Config,
    token: &str,
    timestamp: &str,
    method: &str,
    path: &str,
    body: &str,
) -> Result<String, String> {
    let body_hash = hex::encode(Sha256::digest(body.as_bytes()));
    let signable = format!("{method}\n{body_hash}\n\n{path}");
    let text = format!("{}{token}{timestamp}{signable}", config.client_id);
    let mut hmac = Hmac::<Sha256>::new_from_slice(config.client_secret.as_bytes())
        .map_err(|_| "Não foi possível assinar a solicitação.")?;
    hmac.update(text.as_bytes());
    Ok(hex::encode_upper(hmac.finalize().into_bytes()))
}

fn response_result(value: Value) -> Result<Value, String> {
    if value.get("success").and_then(Value::as_bool) != Some(true) {
        let code = value.get("code").map(|v| v.to_string()).unwrap_or_default();
        // Never return response bodies, tokens or secrets to the frontend/log.
        let hint = match code.trim_matches('"') {
            "1004" => "Assinatura inválida. Confira as chaves e o relógio do Windows.",
            "1010" | "1011" => "A autorização expirou. Atualize a conexão.",
            "1106" | "2007" => "Sem permissão. Confira o vínculo Smart Life, o data center e as APIs autorizadas.",
            "28841002" => "Confira a validade e os limites do plano de API Tuya.",
            _ => "A Tuya recusou a solicitação. Confira região, chaves, dispositivo e serviços autorizados.",
        };
        return Err(hint.into());
    }
    value
        .get("result")
        .cloned()
        .ok_or("Resposta Tuya sem resultado.".into())
}

struct Tuya<'a> {
    config: &'a Config,
    client: Client,
    token: String,
}

impl<'a> Tuya<'a> {
    fn connect(config: &'a Config) -> Result<Self, String> {
        config.validate()?;
        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "Não foi possível iniciar a conexão HTTPS.")?;
        let mut api = Self {
            config,
            client,
            token: String::new(),
        };
        let result = api.request(Method::GET, "/v1.0/token?grant_type=1", "")?;
        api.token = result
            .get("access_token")
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .ok_or("A Tuya não forneceu um token válido.")?
            .to_owned();
        Ok(api)
    }

    fn path(&self, suffix: &str) -> String {
        format!("/v1.0/iot-03/devices/{}{suffix}", self.config.device_id)
    }

    fn request(&self, method: Method, path: &str, body: &str) -> Result<Value, String> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "Confira a data e a hora do computador.")?
            .as_millis()
            .to_string();
        let sign = signature(
            self.config,
            &self.token,
            &timestamp,
            method.as_str(),
            path,
            body,
        )?;
        let mut request = self
            .client
            .request(method, format!("{}{path}", endpoint(&self.config.region)?))
            .header("client_id", &self.config.client_id)
            .header("t", &timestamp)
            .header("sign_method", "HMAC-SHA256")
            .header("sign", sign);
        if !self.token.is_empty() {
            request = request.header("access_token", &self.token);
        }
        if !body.is_empty() {
            request = request
                .header("Content-Type", "application/json")
                .body(body.to_owned());
        }
        let response = request
            .send()
            .map_err(|_| "Não foi possível acessar a Tuya. Confira a internet.")?;
        if !response.status().is_success() {
            return Err("A Tuya retornou um erro HTTP. Confira a conexão e as permissões.".into());
        }
        let value: Value = response.json().map_err(|_| "Resposta Tuya inválida.")?;
        response_result(value)
    }

    fn relay(&self) -> Result<String, String> {
        let functions = self.request(Method::GET, &self.path("/functions"), "")?;
        choose_relay(&functions, &self.config.relay_code)
    }

    fn status(&self, relay: &str) -> Result<LightStatus, String> {
        let info = self.request(Method::GET, &self.path(""), "")?;
        let online = info.get("online").and_then(Value::as_bool);
        let on = if online == Some(true) {
            let values = self.request(Method::GET, &self.path("/status"), "")?;
            read_power(&values, relay)
        } else {
            None
        };
        Ok(LightStatus {
            configured: true,
            online,
            on,
            device_name: info.get("name").and_then(Value::as_str).map(str::to_owned),
            relay_code: Some(relay.to_owned()),
        })
    }
}

fn choose_relay(functions: &Value, requested: &str) -> Result<String, String> {
    let candidates: Vec<&str> = functions
        .get("functions")
        .and_then(Value::as_array)
        .ok_or("A Tuya não retornou as funções da tomada.")?
        .iter()
        .filter_map(|item| {
            let code = item.get("code")?.as_str()?;
            let kind = item.get("type")?.as_str()?;
            if code.starts_with("switch") && kind.eq_ignore_ascii_case("boolean") {
                Some(code)
            } else {
                None
            }
        })
        .collect();
    if !requested.is_empty() {
        return candidates
            .iter()
            .find(|&&code| code == requested)
            .map(|code| (*code).to_owned())
            .ok_or("O canal informado não é um interruptor desta tomada.".into());
    }
    if candidates.len() != 1 {
        return Err("Não foi possível escolher um único interruptor. Confira o canal nas funções da tomada no projeto Tuya.".into());
    }
    Ok(candidates[0].to_owned())
}

fn read_power(values: &Value, relay: &str) -> Option<bool> {
    values
        .as_array()?
        .iter()
        .find(|entry| entry.get("code").and_then(Value::as_str) == Some(relay))?
        .get("value")?
        .as_bool()
}

async fn blocking<T: Send + 'static>(
    action: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(action)
        .await
        .map_err(|_| "Não foi possível concluir a operação Tuya.".to_owned())?
}

#[tauri::command]
pub async fn tuya_connection() -> Result<bool, String> {
    blocking(|| Ok(load_config()?.is_some())).await
}

#[tauri::command]
pub async fn connect_tuya(mut config: Config) -> Result<LightStatus, String> {
    blocking(move || {
        config.validate()?;
        let status = {
            let api = Tuya::connect(&config)?;
            let relay = api.relay()?;
            api.status(&relay)?
        };
        config.relay_code = status.relay_code.clone().unwrap_or_default();
        store_config(&config)?;
        Ok(status)
    })
    .await
}

#[tauri::command]
pub async fn tuya_status() -> Result<LightStatus, String> {
    blocking(|| {
        let Some(config) = load_config()? else {
            return Ok(LightStatus::disconnected());
        };
        let api = Tuya::connect(&config)?;
        let relay = api.relay()?;
        api.status(&relay)
    })
    .await
}

#[tauri::command]
pub async fn set_tuya_power(on: bool) -> Result<LightStatus, String> {
    blocking(move || {
        let config = load_config()?.ok_or("Conecte a tomada Smart Life primeiro.")?;
        let api = Tuya::connect(&config)?;
        let relay = api.relay()?;
        let before = api.status(&relay)?;
        if before.online != Some(true) {
            return Err("A tomada não está online na Tuya.".into());
        }
        let body = json!({ "commands": [{ "code": relay, "value": on }] }).to_string();
        let accepted = api.request(Method::POST, &api.path("/commands"), &body)?;
        if accepted != Value::Bool(true) {
            return Err("A Tuya não aceitou o comando da tomada.".into());
        }
        // A command acknowledgement is not proof of a reported power change.
        let mut status = api.status(&relay)?;
        for _ in 0..2 {
            if status.on == Some(on) || status.online != Some(true) {
                break;
            }
            std::thread::sleep(Duration::from_millis(400));
            status = api.status(&relay)?;
        }
        Ok(status)
    })
    .await
}

#[tauri::command]
pub async fn disconnect_tuya() -> Result<(), String> {
    blocking(|| match credential()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Não foi possível remover a conexão do cofre do sistema.".into()),
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        Config {
            region: "us".into(),
            client_id: "test-client".into(),
            client_secret: "test-secret".into(),
            device_id: "test-device".into(),
            relay_code: "".into(),
        }
    }

    #[test]
    fn only_known_https_regions_are_allowed() {
        assert_eq!(endpoint("us").unwrap(), "https://openapi.tuyaus.com");
        assert!(endpoint("https://example.org").is_err());
        assert!(endpoint("").is_err());
    }

    #[test]
    fn device_identifier_cannot_change_request_path() {
        let mut c = config();
        c.device_id = "../other?secret=1".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn detects_single_boolean_relay_without_guessing_other_functions() {
        let functions = json!({"functions":[
            {"code":"child_lock","type":"Boolean"}, {"code":"switch_1","type":"Boolean"}
        ]});
        assert_eq!(choose_relay(&functions, "").unwrap(), "switch_1");
        assert!(choose_relay(&functions, "child_lock").is_err());
    }

    #[test]
    fn multiple_outlets_require_explicit_selection() {
        let functions = json!({"functions":[
            {"code":"switch_1","type":"Boolean"}, {"code":"switch_2","type":"Boolean"}
        ]});
        assert!(choose_relay(&functions, "").is_err());
        assert_eq!(choose_relay(&functions, "switch_2").unwrap(), "switch_2");
    }

    #[test]
    fn power_is_unknown_for_missing_or_non_boolean_status() {
        assert_eq!(
            read_power(&json!([{"code":"switch_1","value":false}]), "switch_1"),
            Some(false)
        );
        assert_eq!(
            read_power(&json!([{"code":"switch_1","value":"true"}]), "switch_1"),
            None
        );
        assert_eq!(read_power(&json!([]), "switch_1"), None);
    }

    #[test]
    fn api_errors_do_not_echo_response_secrets() {
        let error =
            response_result(json!({"success":false,"code":1106,"msg":"private token test-secret"}))
                .unwrap_err();
        assert!(!error.contains("test-secret"));
        assert!(!error.contains("private token"));
    }

    #[test]
    fn signature_covers_token_body_method_and_path() {
        let c = config();
        let signature_a =
            signature(&c, "access-token", "1700000000000", "POST", "/test", "{}").unwrap();
        assert_eq!(
            signature_a,
            "13F2034D94F7C2573B823AADA8A7FBF19FE91DEC5FA32D2EE87B4506D04DB81C"
        );
        for (token, method, path, body) in [
            ("", "POST", "/test", "{}"),
            ("access-token", "GET", "/test", "{}"),
            ("access-token", "POST", "/other", "{}"),
            ("access-token", "POST", "/test", "[]"),
        ] {
            assert_ne!(
                signature_a,
                signature(&c, token, "1700000000000", method, path, body).unwrap()
            );
        }
    }
}

#[tauri::command]
pub fn open_tuya_guide(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(
            "https://developer.tuya.com/en/docs/developer/apply-cloud-api-key?id=Kff30z8sv62ah",
            None::<&str>,
        )
        .map_err(|_| "Não foi possível abrir o guia no navegador.".into())
}
