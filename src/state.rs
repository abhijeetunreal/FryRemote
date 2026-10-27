//! State shared between the async tasks and the UI.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tv {
    pub name: String,
    pub host: String,
    #[serde(default)]
    pub paired: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConnStatus {
    Idle,
    Connecting,
    Connected,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PairingUi {
    None,
    Starting,
    WaitingForCode,
    WrongCode,
    Verifying,
    Done,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct Shared {
    pub status: ConnStatus,
    pub powered: Option<bool>,
    /// (level, max, muted)
    pub volume: Option<(i32, i32, bool)>,
    pub current_app: Option<String>,
    pub discovered: Vec<Tv>,
    pub scanning: bool,
    pub scan_completed: bool,
    pub pairing: PairingUi,
    /// Set when pairing succeeds so the UI can mark the TV as paired.
    pub paired_host: Option<String>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            status: ConnStatus::Idle,
            powered: None,
            volume: None,
            current_app: None,
            discovered: vec![],
            scanning: false,
            scan_completed: false,
            pairing: PairingUi::None,
            paired_host: None,
        }
    }
}
