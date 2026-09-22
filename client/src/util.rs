use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

/// Best-effort detection of this machine's LAN IPv4 address, so the UI can show a
/// URL that a phone on the same network can open.
///
/// Enumerates local interfaces and keeps private-range IPv4 addresses on
/// interfaces that are not loopback and not VPN-style (`tun*`, `wg*`, ...). The
/// default-route trick (`connect` on a UDP socket) is deliberately avoided: with
/// a VPN active it reports the tunnel address, which is unreachable from other
/// devices on the LAN.
///
/// When several candidates remain, the most "home LAN"-looking one wins:
/// `192.168/16` first, then `172.16/12`, then `10/8`.
pub fn detect_local_ip() -> Option<IpAddr> {
    let mut candidates: Vec<Ipv4Addr> = if_addrs::get_if_addrs()
        .ok()?
        .into_iter()
        .filter(|iface| !iface.is_loopback() && !is_vpn_iface(&iface.name))
        .filter_map(|iface| match iface.ip() {
            IpAddr::V4(ip) if ip.is_private() => Some(ip),
            _ => None,
        })
        .collect();

    candidates.sort_by_key(rank);
    candidates.into_iter().next().map(IpAddr::V4)
}

fn rank(ip: &Ipv4Addr) -> u8 {
    match ip.octets() {
        [192, 168, ..] => 0,
        [172, 16..=31, ..] => 1,
        _ => 2,
    }
}

fn is_vpn_iface(name: &str) -> bool {
    const PREFIXES: [&str; 6] = ["tun", "tap", "wg", "utun", "ppp", "ipsec"];
    PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderPath {
    pub prefix: String,
    pub name: String,
}

pub fn display_path(path: &Path) -> FolderPath {
    let rendered = render_with_tilde(path);

    match rendered.rfind('/') {
        Some(idx) => FolderPath {
            prefix: rendered[..=idx].to_string(),
            name: rendered[idx + 1..].to_string(),
        },
        None => FolderPath {
            prefix: String::new(),
            name: rendered,
        },
    }
}

fn render_with_tilde(path: &Path) -> String {
    let Some(home) = dirs::home_dir() else {
        return path.display().to_string();
    };

    match path.strip_prefix(&home) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn ranks_home_lan_ahead_of_carrier_grade() {
        let mut ips = [
            Ipv4Addr::new(10, 8, 1, 2),
            Ipv4Addr::new(192, 168, 0, 112),
            Ipv4Addr::new(172, 20, 5, 5),
        ];
        ips.sort_by_key(rank);
        assert_eq!(ips[0], Ipv4Addr::new(192, 168, 0, 112));
        assert_eq!(ips[1], Ipv4Addr::new(172, 20, 5, 5));
    }

    #[test]
    fn recognizes_tunnel_interface_names() {
        assert!(is_vpn_iface("tun0"));
        assert!(is_vpn_iface("wg0"));
        assert!(is_vpn_iface("utun3"));
        assert!(!is_vpn_iface("wlo1"));
        assert!(!is_vpn_iface("eth0"));
    }

    #[test]
    fn replaces_home_dir_with_tilde() {
        let home = dirs::home_dir().expect("home dir must be set for this test");
        assert_eq!(
            display_path(&home),
            FolderPath {
                prefix: String::new(),
                name: "~".to_string(),
            }
        );
        assert_eq!(
            display_path(&home.join("Documents")),
            FolderPath {
                prefix: "~/".to_string(),
                name: "Documents".to_string(),
            }
        );
    }

    #[test]
    fn leaves_non_home_paths_untouched() {
        let path = PathBuf::from("/srv/shared");
        assert_eq!(
            display_path(&path),
            FolderPath {
                prefix: "/srv/".to_string(),
                name: "shared".to_string(),
            }
        );
    }
}
