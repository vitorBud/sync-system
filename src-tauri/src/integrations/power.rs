use std::net::{Ipv4Addr, UdpSocket};
fn magic_packet(mac: &str) -> Result<[u8; 102], String> {
    let separator = if mac.contains(':') { ':' } else { '-' };
    let parts: Vec<_> = mac.split(separator).collect();
    if parts.len() != 6 || parts.iter().any(|part| part.len() != 2) {
        return Err("Informe um MAC com seis pares hexadecimais.".into());
    }
    let mut address = [0u8; 6];
    for (index, part) in parts.iter().enumerate() {
        address[index] = u8::from_str_radix(part, 16).map_err(|_| "MAC inválido.")?;
    }
    if address == [0; 6] || address[0] & 1 != 0 {
        return Err("Informe o MAC individual da placa de rede.".into());
    }
    let mut packet = [0xff; 102];
    for chunk in packet[6..].chunks_mut(6) {
        chunk.copy_from_slice(&address);
    }
    Ok(packet)
}

fn wake_destination(value: &str) -> Result<Ipv4Addr, String> {
    let address: Ipv4Addr = value.parse().map_err(|_| "Endereço IPv4 inválido.")?;
    if !address.is_private() && !address.is_broadcast() {
        return Err("Use o broadcast da sua rede local.".into());
    }
    Ok(address)
}

pub fn validate_wake(mac: &str, broadcast: &str) -> Result<(), String> {
    magic_packet(mac)?;
    wake_destination(broadcast)?;
    Ok(())
}

pub fn wake(mac: &str, broadcast: &str) -> Result<(), String> {
    let packet = magic_packet(mac)?;
    let target = wake_destination(broadcast)?;

    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| e.to_string())?;
    socket.set_broadcast(true).map_err(|e| e.to_string())?;
    socket
        .send_to(&packet, (target, 9))
        .map_err(|e| e.to_string())?;
    Ok::<_, String>(())
}

pub fn shutdown_local(confirmed: bool) -> Result<(), String> {
    if !confirmed {
        return Err("O desligamento exige confirmação.".into());
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let system_root = std::env::var_os("SystemRoot").ok_or("Windows não localizado.")?;
        let output = std::process::Command::new(
            std::path::PathBuf::from(system_root)
                .join("System32")
                .join("shutdown.exe"),
        )
        .args(["/s", "/t", "0"])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("O Windows não aceitou o desligamento.".into());
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    Err("Desligamento local disponível apenas no Windows nesta etapa.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wol_contains_sync_header_and_sixteen_mac_copies() {
        let packet = magic_packet("02:11:22:33:44:55").unwrap();
        assert_eq!(&packet[..6], &[255; 6]);
        for chunk in packet[6..].chunks(6) {
            assert_eq!(chunk, &[2, 17, 34, 51, 68, 85]);
        }
        assert_eq!(packet, magic_packet("02-11-22-33-44-55").unwrap());
    }

    #[test]
    fn rejects_invalid_or_non_individual_mac_addresses() {
        for mac in [
            "",
            "00:00:00:00:00:00",
            "FF:FF:FF:FF:FF:FF",
            "01:11:22:33:44:55",
            "02:11:22",
            "GG:11:22:33:44:55",
        ] {
            assert!(magic_packet(mac).is_err(), "{mac}");
        }
    }

    #[test]
    fn wake_is_limited_to_local_network_destinations() {
        assert!(wake_destination("192.168.1.255").is_ok());
        assert!(wake_destination("255.255.255.255").is_ok());
        for address in ["127.0.0.1", "8.8.8.8", "example.com", "not-an-ip"] {
            assert!(wake_destination(address).is_err());
        }
    }

    #[test]
    fn shutdown_without_confirmation_never_runs() {
        assert!(shutdown_local(false).is_err());
    }
}
