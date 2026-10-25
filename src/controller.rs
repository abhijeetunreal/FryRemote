//! Glue between the UI and the async networking code.

use crate::creds::Creds;
use crate::discovery;
use crate::pairing::{CodeError, PairingSession};
use crate::remote::{self, RemoteHandle};
use crate::state::*;
use crate::store::Store;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub type Notify = Arc<dyn Fn() + Send + Sync>;

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

fn set_pairing(shared: &Arc<Mutex<Shared>>, notify: &Notify, p: PairingUi) {
    shared.lock().unwrap().pairing = p;
    notify();
}
