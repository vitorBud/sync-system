use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WakeTarget {
    pub mac: String,
    pub broadcast: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: u8,
    pub wake_target: Option<WakeTarget>,
    pub sunshine_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            wake_target: None,
            sunshine_port: 47989,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Versão de configuração não suportada.".into());
        }
        if !(1024..=65534).contains(&self.sunshine_port) {
            return Err("A porta do Sunshine deve estar entre 1024 e 65534.".into());
        }
        if let Some(target) = &self.wake_target {
            crate::integrations::power::validate_wake(&target.mac, &target.broadcast)?;
        }
        Ok(())
    }
}

pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Settings, String> {
        match fs::read_to_string(&self.path) {
            Ok(text) => {
                let settings: Settings = serde_json::from_str(&text)
                    .map_err(|_| "Configuração inválida; o arquivo original foi preservado.")?;
                settings.validate()?;
                Ok(settings)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
            Err(_) => Err("Não foi possível ler a configuração deste computador.".into()),
        }
    }

    pub fn save(&self, settings: &Settings) -> Result<(), String> {
        settings.validate()?;
        let parent = self
            .path
            .parent()
            .ok_or("Pasta de configuração inválida.")?;
        fs::create_dir_all(parent)
            .map_err(|_| "Não foi possível criar a pasta de configuração.")?;
        let bytes = serde_json::to_vec_pretty(settings).map_err(|_| "Configuração inválida.")?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)
            .map_err(|_| "Não foi possível preparar a configuração.")?;
        temp.write_all(&bytes)
            .map_err(|_| "Não foi possível gravar a configuração.")?;
        temp.as_file()
            .sync_all()
            .map_err(|_| "Não foi possível concluir a gravação.")?;
        temp.persist(&self.path)
            .map_err(|_| "Não foi possível substituir a configuração.")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_settings_have_no_fake_wake_target() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("controls.json"));
        let settings = store.load().unwrap();
        assert!(settings.wake_target.is_none());
        assert_eq!(settings.sunshine_port, 47989);
    }

    #[test]
    fn invalid_save_preserves_existing_settings() {
        let dir = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(dir.path().join("controls.json"));
        let mut settings = Settings::default();
        settings.sunshine_port = 48000;
        store.save(&settings).unwrap();
        let mut invalid = settings.clone();
        invalid.sunshine_port = 65535;
        assert!(store.save(&invalid).is_err());
        assert_eq!(store.load().unwrap().sunshine_port, 48000);
        settings.sunshine_port = 47989;
        store.save(&settings).unwrap();
        assert_eq!(store.load().unwrap().sunshine_port, 47989);
    }

    #[test]
    fn corrupt_settings_are_not_replaced_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("controls.json");
        fs::write(&path, b"broken").unwrap();
        assert!(SettingsStore::new(path.clone()).load().is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "broken");
    }
}
