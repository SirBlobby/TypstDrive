use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

fn allows_private_addresses() -> bool {
    std::env::var("REMOTE_URLS_ALLOW_PRIVATE").map(|value| value.eq_ignore_ascii_case("true") || value == "1").unwrap_or(false)
}

fn is_public_address(address: &IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let [first, second, ..] = address.octets();
            let shared_address_space = first == 100 && (second & 0xc0) == 64;
            !(address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_broadcast()
                || address.is_documentation()
                || address.is_unspecified()
                || address.is_multicast()
                || first == 0
                || first >= 240
                || shared_address_space)
        }
        IpAddr::V6(address) => {
            if let Some(mapped) = address.to_ipv4_mapped() {
                return is_public_address(&IpAddr::V4(mapped));
            }
            let first_segment = address.segments()[0];
            let unique_local = (first_segment & 0xfe00) == 0xfc00;
            let link_local = (first_segment & 0xffc0) == 0xfe80;
            !(address.is_loopback() || address.is_unspecified() || address.is_multicast() || unique_local || link_local)
        }
    }
}

pub fn resolve_remote_host(host_and_port: &str) -> std::io::Result<Vec<SocketAddr>> {
    let addresses: Vec<SocketAddr> = host_and_port.to_socket_addrs()?.collect();
    if allows_private_addresses() {
        return Ok(addresses);
    }
    let public: Vec<SocketAddr> = addresses.into_iter().filter(|address| is_public_address(&address.ip())).collect();
    if public.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "remote files must be on a public address",
        ));
    }
    Ok(public)
}
