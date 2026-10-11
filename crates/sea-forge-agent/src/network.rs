use crate::config::EndpointSnapshot;
use reqwest::{redirect::Policy, Client};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use url::Url;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkPolicy {
    pub allow_loopback_test: bool,
}

pub fn validate_destination(url: &Url, policy: NetworkPolicy) -> Result<Vec<SocketAddr>, String> {
    let host = url
        .host_str()
        .ok_or_else(|| "endpoint URL has no host".to_string())?;
    if url.scheme() != "https"
        && !(policy.allow_loopback_test && url.scheme() == "http" && is_loopback(host))
    {
        return Err("production agent endpoints require HTTPS".into());
    }
    let port = url
        .port_or_known_default()
        .ok_or_else(|| "endpoint URL has no known port".to_string())?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("endpoint DNS resolution failed: {e}"))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err("endpoint DNS resolution returned no addresses".into());
    }
    if !policy.allow_loopback_test && addresses.iter().any(|address| !is_public(address.ip())) {
        return Err("endpoint resolves to a private or reserved destination".into());
    }
    if policy.allow_loopback_test
        && addresses
            .iter()
            .any(|address| !address.ip().is_loopback() && !is_public(address.ip()))
    {
        return Err("test endpoint resolves to a non-loopback private destination".into());
    }
    Ok(addresses)
}

pub fn prepare_client(endpoint: &EndpointSnapshot) -> Result<(Client, Url), String> {
    let addresses = validate_destination(
        &endpoint.base_url,
        NetworkPolicy {
            allow_loopback_test: endpoint.allow_loopback_test,
        },
    )?;
    let host = endpoint
        .base_url
        .host_str()
        .ok_or_else(|| "endpoint URL has no host".to_string())?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .https_only(endpoint.base_url.scheme() == "https")
        .timeout(endpoint.timeout)
        .resolve_to_addrs(host, &addresses)
        .build()
        .map_err(|e| format!("build HTTP client: {e}"))?;
    Ok((client, endpoint.base_url.clone()))
}

fn is_loopback(host: &str) -> bool {
    host == "localhost" || host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            !(octets[0] == 10
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 168)
                || octets[0] == 127
                || (octets[0] == 169 && octets[1] == 254)
                || octets[0] >= 224
                || ip.is_unspecified())
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            !(ip.is_loopback()
                || ip.is_unspecified()
                || (segments[0] & 0xfe00) == 0xfc00
                || (segments[0] & 0xffc0) == 0xfe80
                || segments[0] == 0xff00)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_and_metadata_addresses() {
        for address in ["10.0.0.1", "192.168.1.1", "169.254.169.254", "127.0.0.1"] {
            let ip = address.parse().unwrap();
            assert!(!is_public(ip), "{address} must not be public");
        }
    }

    #[test]
    fn loopback_http_requires_explicit_test_policy() {
        let url = Url::parse("http://127.0.0.1:8080/").unwrap();
        assert!(validate_destination(
            &url,
            NetworkPolicy {
                allow_loopback_test: false
            }
        )
        .is_err());
    }
}
