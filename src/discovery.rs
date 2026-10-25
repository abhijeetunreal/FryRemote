//! Find TVs on the local network via mDNS (`_androidtvremote2._tcp`).

use crate::state::{Shared, Tv};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SERVICE: &str = "_androidtvremote2._tcp.local.";

/// Scan for `seconds`, adding results to `shared.discovered`.
/// Blocking: call from a dedicated thread.
pub fn scan(shared: Arc<Mutex<Shared>>, notify: Arc<dyn Fn() + Send + Sync>, seconds: u64) {
    {
        let mut s = shared.lock().unwrap();
        s.scanning = true;
        s.discovered.clear();
    }
    notify();

    match ServiceDaemon::new().and_then(|d| d.browse(SERVICE).map(|r| (d, r))) {
        Ok((daemon, receiver)) => {
            let deadline = Instant::now() + Duration::from_secs(seconds);
            while Instant::now() < deadline {
                match receiver.recv_timeout(Duration::from_millis(500)) {
                    Ok(ServiceEvent::ServiceResolved(info)) => {
                        let fullname = info.get_fullname().to_string();
                        let name = fullname
                            .strip_suffix(&format!(".{SERVICE}"))
                            .unwrap_or(&fullname)
                            .to_string();
                        // Prefer IPv4: the pairing/remote sockets are plain TCP on the LAN.
                        let ip = info
                            .get_addresses()
                            .iter()
                            .map(|a| a.to_ip_addr())
                            .find(|a| a.is_ipv4())
                            .or_else(|| info.get_addresses().iter().map(|a| a.to_ip_addr()).next());
                        if let Some(ip) = ip {
                            let tv = Tv { name, host: ip.to_string(), paired: false };
                            let mut s = shared.lock().unwrap();
                            if !s.discovered.iter().any(|t| t.host == tv.host) {
                                s.discovered.push(tv);
                            }
                            drop(s);
                            notify();
                        }
                    }
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
            let _ = daemon.shutdown();
        }
        Err(e) => log::warn!("mDNS unavailable: {e}"),
    }

    shared.lock().unwrap().scanning = false;
    notify();
}
