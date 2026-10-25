//! Remote-control session (TV port 6466): keeps one connection alive,
//! answers pings, injects keys and launches apps.

use crate::creds::Creds;
use crate::proto::*;
use crate::state::{ConnStatus, Shared};
use crate::tls;
use crate::wire::{read_msg, write_msg};
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

pub const REMOTE_PORT: u16 = 6466;

#[derive(Debug)]
pub enum Cmd {
    Key { code: i32, direction: i32 },
    Launch(String),
    Disconnect,
}

#[derive(Clone)]
pub struct RemoteHandle {
    tx: mpsc::UnboundedSender<Cmd>,
}

impl RemoteHandle {
    pub fn key(&self, code: i32) {
        let _ = self.tx.send(Cmd::Key { code, direction: DIR_SHORT });
    }
    pub fn key_dir(&self, code: i32, direction: i32) {
        let _ = self.tx.send(Cmd::Key { code, direction });
    }
    pub fn launch(&self, link: &str) {
        let _ = self.tx.send(Cmd::Launch(link.to_string()));
    }
    pub fn disconnect(&self) {
        let _ = self.tx.send(Cmd::Disconnect);
    }
}

type Notify = Arc<dyn Fn() + Send + Sync>;

/// Start a background task that connects (and reconnects) to the TV.
pub fn spawn(
    rt: &tokio::runtime::Handle,
    host: String,
    creds: Arc<Creds>,
    shared: Arc<Mutex<Shared>>,
    notify: Notify,
) -> RemoteHandle {
    spawn_on(rt, host, REMOTE_PORT, creds, shared, notify)
}

pub fn spawn_on(
    rt: &tokio::runtime::Handle,
    host: String,
    port: u16,
    creds: Arc<Creds>,
    shared: Arc<Mutex<Shared>>,
    notify: Notify,
) -> RemoteHandle {
    let (tx, rx) = mpsc::unbounded_channel();
    rt.spawn(run(host, port, creds, shared, notify, rx));
    RemoteHandle { tx }
}

fn set_status(shared: &Arc<Mutex<Shared>>, notify: &Notify, s: ConnStatus) {
    shared.lock().unwrap().status = s;
    notify();
}

async fn run(
    host: String,
    port: u16,
    creds: Arc<Creds>,
    shared: Arc<Mutex<Shared>>,
    notify: Notify,
    mut rx: mpsc::UnboundedReceiver<Cmd>,
) {
    let mut failures = 0u32;
    loop {
        set_status(&shared, &notify, ConnStatus::Connecting);
        // Drop keys queued while we were offline; honour a pending disconnect.
        while let Ok(c) = rx.try_recv() {
            if matches!(c, Cmd::Disconnect) {
                set_status(&shared, &notify, ConnStatus::Idle);
                return;
            }
        }
        match session(&host, port, &creds, &shared, &notify, &mut rx).await {
            Ok(End::UserClosed) => {
                set_status(&shared, &notify, ConnStatus::Idle);
                return;
            }
            Ok(End::Dropped { was_ready }) => {
                failures = if was_ready { 0 } else { failures + 1 };
                log::warn!("remote connection dropped (ready={was_ready})");
            }
            Err(e) => {
                failures += 1;
                log::warn!("remote session error: {e:#}");
                if failures >= 3 {
                    set_status(
                        &shared,
                        &notify,
                        ConnStatus::Failed(format!(
                            "{e:#}. If this TV was never paired, use Pair."
                        )),
                    );
                    return;
                }
            }
        }
        if failures >= 3 {
            set_status(
                &shared,
                &notify,
                ConnStatus::Failed("The TV keeps closing the connection. Try pairing again.".into()),
            );
            return;
        }
        tokio::time::sleep(Duration::from_secs((1 << failures.min(3)) as u64)).await;
    }
}

enum End {
    UserClosed,
    Dropped { was_ready: bool },
}

async fn session(
    host: &str,
    port: u16,
    creds: &Creds,
    shared: &Arc<Mutex<Shared>>,
    notify: &Notify,
    rx: &mut mpsc::UnboundedReceiver<Cmd>,
) -> Result<End> {
    let stream = tls::connect(host, port, creds).await?;
    let (mut rd, mut wr) = tokio::io::split(stream);

    // Reader task, so reads are never cancelled half-way through a message.
    let (mtx, mut mrx) = mpsc::unbounded_channel::<Result<RemoteMessage>>();
    let reader = tokio::spawn(async move {
        loop {
            let m = read_msg::<_, RemoteMessage>(&mut rd).await;
            let stop = m.is_err();
            if mtx.send(m).is_err() || stop {
                break;
            }
        }
    });

    let mut ready = false;
    let result = loop {
        tokio::select! {
            msg = mrx.recv() => {
                let Some(msg) = msg else { break Ok(End::Dropped { was_ready: ready }) };
                let msg = match msg {
                    Ok(m) => m,
                    Err(e) => {
                        break if ready { Ok(End::Dropped { was_ready: true }) } else {
                            Err(anyhow!("TV closed the connection before it was ready ({e})"))
                        };
                    }
                };
                if let Some(_cfg) = msg.configure {
                    let reply = RemoteMessage {
                        configure: Some(RemoteConfigure {
                            code1: REMOTE_FEATURES,
                            device_info: Some(RemoteDeviceInfo {
                                model: "FryRemote".into(),
                                vendor: "Rust".into(),
                                unknown1: 1,
                                unknown2: "1".into(),
                                package_name: "dev.fryremote.app".into(),
                                app_version: "1.0.0".into(),
                            }),
                        }),
                        ..Default::default()
                    };
                    write_msg(&mut wr, &reply).await?;
                }
                if msg.set_active.is_some() {
                    let reply = RemoteMessage {
                        set_active: Some(RemoteSetActive { active: REMOTE_FEATURES }),
                        ..Default::default()
                    };
                    write_msg(&mut wr, &reply).await?;
                    ready = true;
                    set_status(shared, notify, ConnStatus::Connected);
                }
                if let Some(p) = msg.ping_request {
                    let reply = RemoteMessage {
                        ping_response: Some(RemotePingResponse { val1: p.val1 }),
                        ..Default::default()
                    };
                    write_msg(&mut wr, &reply).await?;
                }
                if let Some(s) = msg.start {
                    shared.lock().unwrap().powered = Some(s.started);
                    notify();
                }
                if let Some(v) = msg.set_volume_level {
                    shared.lock().unwrap().volume = Some((v.volume_level, v.volume_max, v.volume_muted));
                    notify();
                }
                if let Some(i) = msg.ime_key_inject {
                    if let Some(a) = i.app_info {
                        shared.lock().unwrap().current_app =
                            Some(a.app_package).filter(|s| !s.is_empty());
                        notify();
                    }
                }
            }
            cmd = rx.recv() => {
                match cmd {
                    None | Some(Cmd::Disconnect) => break Ok(End::UserClosed),
                    Some(Cmd::Key { code, direction }) => {
                        if ready {
                            let m = RemoteMessage {
                                key_inject: Some(RemoteKeyInject { key_code: code, direction }),
                                ..Default::default()
                            };
                            write_msg(&mut wr, &m).await?;
                        }
                    }
                    Some(Cmd::Launch(link)) => {
                        if ready {
                            let m = RemoteMessage {
                                app_link_launch: Some(RemoteAppLinkLaunchRequest { app_link: link }),
                                ..Default::default()
                            };
                            write_msg(&mut wr, &m).await?;
                        }
                    }
                }
            }
        }
    };
    reader.abort();
    result
}
