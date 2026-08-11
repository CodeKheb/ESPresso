use std::collections::HashSet;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};

use crate::models::DiscoveredDevice;

const SERVICE_TYPES: &[&str] = &["_espresso._tcp.local.", "_http._tcp.local."];

/// Long-lived mDNS browser. One `ServiceDaemon` is created once and reused for
/// every scan — creating/shutting down a daemon per scan leaks resources and
/// eventually breaks discovery.
pub struct MdnsBrowser {
    daemon: Option<ServiceDaemon>,
}

impl MdnsBrowser {
    pub fn new() -> Self {
        match ServiceDaemon::new() {
            Ok(d) => Self { daemon: Some(d) },
            Err(err) => {
                eprintln!("[mdns] failed to start service daemon: {err}");
                Self { daemon: None }
            }
        }
    }

    /// Browses for ESPresso pots. Hostnames/instances in `exclude` (our own
    /// advertisement) are skipped. Returns an empty list on any failure.
    pub fn browse(&self, timeout: Duration, exclude: &[String]) -> Vec<DiscoveredDevice> {
        let Some(daemon) = &self.daemon else {
            return vec![];
        };

        let mut found: Vec<DiscoveredDevice> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // Share the budget across service types so the total stays bounded.
        let per_service = timeout / SERVICE_TYPES.len() as u32;

        for service in SERVICE_TYPES {
            let rx = match daemon.browse(service) {
                Ok(rx) => rx,
                Err(err) => {
                    eprintln!("[mdns] browse {service} failed: {err}");
                    continue;
                }
            };

            let start = Instant::now();
            loop {
                let elapsed = start.elapsed();
                if elapsed >= per_service {
                    break;
                }
                match rx.recv_timeout(per_service - elapsed) {
                    Ok(ServiceEvent::ServiceResolved(info)) => {
                        let hostname = info.get_hostname().trim_end_matches('.').to_lowercase();
                        let fullname = info.get_fullname().to_lowercase();
                        // Skip our own advertisement.
                        if exclude.iter().any(|e| {
                            let e = e.to_lowercase();
                            hostname == e || fullname.starts_with(&format!("{e}."))
                        }) {
                            continue;
                        }
                        // `_espresso` is ours by definition; for generic HTTP
                        // services accept hostnames that mention the device.
                        let is_espresso = service.starts_with("_espresso")
                            || hostname.contains("espresso")
                            || hostname.starts_with("esp32")
                            || hostname.starts_with("espressif")
                            || fullname.contains("espresso");
                        if !is_espresso {
                            continue;
                        }
                        for ip in info.get_addresses() {
                            // Only v4 for now: the WS URL builder can't
                            // represent link-local IPv6 without brackets.
                            if !ip.is_ipv4() {
                                continue;
                            }
                            let key = format!("{ip}:{}", info.get_port());
                            if seen.insert(key) {
                                found.push(DiscoveredDevice {
                                    name: hostname.clone(),
                                    host: hostname.clone(),
                                    ip: Some(ip.to_string()),
                                    port: info.get_port(),
                                });
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        }

        found
    }
}

impl Default for MdnsBrowser {
    fn default() -> Self {
        Self::new()
    }
}

/// Advertises this device's pot as `_espresso._tcp.local.` so other instances
/// on the same WiFi can find us. Returns the daemon — it must be kept alive
/// for the advertisement to persist.
pub fn advertise(hostname: &str, instance: &str, port: u16) -> Option<ServiceDaemon> {
    let daemon = ServiceDaemon::new().ok()?;
    let ips: Vec<IpAddr> = if_addrs::get_if_addrs()
        .ok()?
        .into_iter()
        .map(|a| a.ip())
        .filter(|ip| !ip.is_loopback())
        .collect();
    let info = ServiceInfo::new(
        "_espresso._tcp.local.",
        instance,
        hostname,
        ips.as_slice(),
        port,
        [("path", "/")].as_slice(),
    )
    .ok()?;
    if daemon.register(info).is_err() {
        return None;
    }
    Some(daemon)
}
