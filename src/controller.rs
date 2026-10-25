//! Glue between the UI and the async networking code.

use crate::state::*;
use crate::store::Store;
use std::sync::{Arc, Mutex};

pub type Notify = Arc<dyn Fn() + Send + Sync>;

#[cfg(not(target_arch = "wasm32"))]
use crate::creds::Creds;
#[cfg(not(target_arch = "wasm32"))]
use crate::discovery;
#[cfg(not(target_arch = "wasm32"))]
use crate::pairing::{CodeError, PairingSession};
#[cfg(not(target_arch = "wasm32"))]
use crate::remote::{self, RemoteHandle};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;

#[cfg(not(target_arch = "wasm32"))]
pub struct Controller {
    rt: tokio::runtime::Runtime,
    pub shared: Arc<Mutex<Shared>>,
    creds: Arc<Mutex<Option<Arc<Creds>>>>,
    creds_error: Arc<Mutex<Option<String>>>,
    pub store: Store,
    remote: Option<RemoteHandle>,
    pub active: Option<Tv>,
    pair_tx: Option<mpsc::UnboundedSender<String>>,
    notify: Notify,
}

#[cfg(not(target_arch = "wasm32"))]
impl Controller {
    pub fn new(data_dir: PathBuf, notify: Notify) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("tokio runtime");
        let store = Store::load(&data_dir);
        let creds = Arc::new(Mutex::new(None));
        let creds_error = Arc::new(Mutex::new(None));

        // First launch generates a 2048-bit RSA identity, which takes a moment.
        {
            let (creds, err, notify) = (creds.clone(), creds_error.clone(), notify.clone());
            std::thread::spawn(move || {
                match Creds::load_or_create(&data_dir) {
                    Ok(c) => *creds.lock().unwrap() = Some(Arc::new(c)),
                    Err(e) => *err.lock().unwrap() = Some(format!("{e:#}")),
                }
                notify();
            });
        }

        Self {
            rt,
            shared: Arc::new(Mutex::new(Shared::default())),
            creds,
            creds_error,
            store,
            remote: None,
            active: None,
            pair_tx: None,
            notify,
        }
    }

    pub fn creds_ready(&self) -> bool {
        self.creds.lock().unwrap().is_some()
    }

    pub fn creds_error(&self) -> Option<String> {
        self.creds_error.lock().unwrap().clone()
    }

    /// Call once per frame.
    pub fn poll(&mut self) {
        let host = self.shared.lock().unwrap().paired_host.take();
        if let Some(h) = host {
            self.store.mark_paired(&h);
        }
    }

    pub fn snapshot(&self) -> Shared {
        self.shared.lock().unwrap().clone()
    }

    // ---- discovery -----------------------------------------------------

    pub fn scan(&self) {
        if self.shared.lock().unwrap().scanning {
            return;
        }
        let (shared, notify) = (self.shared.clone(), self.notify.clone());
        std::thread::spawn(move || discovery::scan(shared, notify, 8));
    }

    // ---- pairing -------------------------------------------------------

    pub fn start_pairing(&mut self, tv: &Tv) {
        self.cancel_pairing();
        let Some(creds) = self.creds.lock().unwrap().clone() else { return };
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        self.pair_tx = Some(tx);
        let (shared, notify, host) = (self.shared.clone(), self.notify.clone(), tv.host.clone());
        set_pairing(&shared, &notify, PairingUi::Starting);

        self.rt.spawn(async move {
            let mut session = match PairingSession::start(&host, &creds, "FryRemote").await {
                Ok(s) => s,
                Err(e) => {
                    set_pairing(&shared, &notify, PairingUi::Error(format!("{e:#}")));
                    return;
                }
            };
            set_pairing(&shared, &notify, PairingUi::WaitingForCode);
            while let Some(code) = rx.recv().await {
                set_pairing(&shared, &notify, PairingUi::Verifying);
                match session.finish(&code).await {
                    Ok(()) => {
                        shared.lock().unwrap().paired_host = Some(host.clone());
                        set_pairing(&shared, &notify, PairingUi::Done);
                        return;
                    }
                    Err(CodeError::WrongCode) => set_pairing(&shared, &notify, PairingUi::WrongCode),
                    Err(CodeError::Other(e)) => {
                        set_pairing(&shared, &notify, PairingUi::Error(format!("{e:#}")));
                        return;
                    }
                }
            }
        });
    }

    pub fn submit_code(&self, code: &str) {
        if let Some(tx) = &self.pair_tx {
            let _ = tx.send(code.to_string());
        }
    }

    pub fn cancel_pairing(&mut self) {
        self.pair_tx = None; // dropping the sender ends the pairing task
        self.shared.lock().unwrap().pairing = PairingUi::None;
    }

    // ---- remote --------------------------------------------------------

    pub fn connect(&mut self, tv: &Tv) {
        self.disconnect();
        let Some(creds) = self.creds.lock().unwrap().clone() else { return };
        {
            let mut s = self.shared.lock().unwrap();
            s.status = ConnStatus::Connecting;
            s.powered = None;
            s.volume = None;
            s.current_app = None;
        }
        self.active = Some(tv.clone());
        self.remote = Some(remote::spawn(
            self.rt.handle(),
            tv.host.clone(),
            creds,
            self.shared.clone(),
            self.notify.clone(),
        ));
    }

    pub fn disconnect(&mut self) {
        if let Some(r) = self.remote.take() {
            r.disconnect();
        }
        self.active = None;
        self.shared.lock().unwrap().status = ConnStatus::Idle;
    }

    pub fn key(&self, code: i32) {
        if let Some(r) = &self.remote {
            r.key(code);
        }
    }

    pub fn send_text(&self, text: &str) {
        if let Some(r) = &self.remote {
            r.text(text);
        }
    }

    pub fn launch(&self, link: &str) {
        if let Some(r) = &self.remote {
            r.launch(link);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn set_pairing(shared: &Arc<Mutex<Shared>>, notify: &Notify, p: PairingUi) {
    shared.lock().unwrap().pairing = p;
    notify();
}

#[cfg(target_arch = "wasm32")]
pub struct Controller {
    pub shared: Arc<Mutex<Shared>>,
    pub store: Store,
    pub active: Option<Tv>,
    notify: Notify,
}

#[cfg(target_arch = "wasm32")]
impl Controller {
    pub fn new_web(notify: Notify) -> Self {
        let mut shared = Shared::default();
        shared.status = ConnStatus::Connected;
        shared.powered = Some(true);
        shared.volume = Some((35, 100, false));
        let store = Store::demo();
        let active = store.tvs.first().cloned();

        Self {
            shared: Arc::new(Mutex::new(shared)),
            store,
            active,
            notify,
        }
    }

    pub fn creds_ready(&self) -> bool {
        true
    }

    pub fn creds_error(&self) -> Option<String> {
        None
    }

    pub fn poll(&mut self) {}

    pub fn snapshot(&self) -> Shared {
        self.shared.lock().unwrap().clone()
    }

    pub fn scan(&self) {
        let (shared, notify) = (self.shared.clone(), self.notify.clone());
        shared.lock().unwrap().scanning = true;
        notify();
        let mut s = shared.lock().unwrap();
        s.scanning = false;
        s.discovered = vec![
            Tv { name: "Living Room Google TV (Demo)".into(), host: "192.168.1.100".into(), paired: true },
            Tv { name: "Bedroom Android TV (Demo)".into(), host: "192.168.1.102".into(), paired: false },
        ];
        notify();
    }

    pub fn start_pairing(&mut self, _tv: &Tv) {
        let mut s = self.shared.lock().unwrap();
        s.pairing = PairingUi::WaitingForCode;
        (self.notify)();
    }

    pub fn submit_code(&self, _code: &str) {
        let mut s = self.shared.lock().unwrap();
        s.pairing = PairingUi::Done;
        (self.notify)();
    }

    pub fn cancel_pairing(&mut self) {
        self.shared.lock().unwrap().pairing = PairingUi::None;
        (self.notify)();
    }

    pub fn connect(&mut self, tv: &Tv) {
        self.active = Some(tv.clone());
        let mut s = self.shared.lock().unwrap();
        s.status = ConnStatus::Connected;
        s.powered = Some(true);
        s.volume = Some((35, 100, false));
        (self.notify)();
    }

    pub fn disconnect(&mut self) {
        self.active = None;
        let mut s = self.shared.lock().unwrap();
        s.status = ConnStatus::Idle;
        (self.notify)();
    }

    pub fn key(&self, code: i32) {
        let mut s = self.shared.lock().unwrap();
        match code {
            crate::keys::POWER => {
                let cur = s.powered.unwrap_or(true);
                s.powered = Some(!cur);
            }
            crate::keys::VOLUME_UP => {
                if let Some((level, max, muted)) = &mut s.volume {
                    *level = (*level + 2).min(*max);
                    *muted = false;
                }
            }
            crate::keys::VOLUME_DOWN => {
                if let Some((level, _max, muted)) = &mut s.volume {
                    *level = (*level - 2).max(0);
                    *muted = false;
                }
            }
            crate::keys::VOLUME_MUTE => {
                if let Some((_level, _max, muted)) = &mut s.volume {
                    *muted = !*muted;
                }
            }
            _ => {}
        }
        (self.notify)();
    }


    pub fn launch(&self, link: &str) {
        let app_name = if link.contains("youtube") {
            "YouTube"
        } else if link.contains("netflix") {
            "Netflix"
        } else if link.contains("prime") {
            "Prime Video"
        } else if link.contains("disney") {
            "Disney+"
        } else if link.contains("hotstar") {
            "Hotstar"
        } else if link.contains("spotify") {
            "Spotify"
        } else {
            "App"
        };
        let mut s = self.shared.lock().unwrap();
        s.current_app = Some(app_name.to_string());
        (self.notify)();
    }

    pub fn send_text(&self, _text: &str) {}
}
