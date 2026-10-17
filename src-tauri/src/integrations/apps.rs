use std::path::PathBuf;

pub fn remote_play_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    for base in ["ProgramFiles(x86)", "ProgramFiles"]
        .iter()
        .filter_map(std::env::var_os)
    {
        let path = PathBuf::from(base)
            .join("Sony")
            .join("PS Remote Play")
            .join("RemotePlay.exe");
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

pub fn launch_remote_play() -> Result<(), String> {
    let path = remote_play_path().ok_or("PS Remote Play não encontrado neste computador.")?;
    std::process::Command::new(&path)
        .current_dir(path.parent().ok_or("Pasta do aplicativo inválida")?)
        .spawn()
        .map_err(|_| "Não foi possível abrir o PS Remote Play.")?;
    Ok(())
}
