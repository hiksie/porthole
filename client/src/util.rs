use std::net::{IpAddr, Ipv4Addr};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
