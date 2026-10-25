//! Protobuf messages for the Android TV Remote Service v2 protocol.
//!
//! Defined with prost derive macros so no `protoc` is needed at build time.

use prost::Message;

// ---------------------------------------------------------------------------
// Pairing protocol (port 6467)
// ---------------------------------------------------------------------------

pub const PAIRING_PROTOCOL_VERSION: i32 = 2;
pub const STATUS_OK: i32 = 200;
pub const ENCODING_HEXADECIMAL: i32 = 3;
pub const ROLE_INPUT: i32 = 1;

#[derive(Clone, PartialEq, Message)]
pub struct PairingMessage {
    #[prost(int32, tag = "1")]
    pub protocol_version: i32,
    #[prost(int32, tag = "2")]
    pub status: i32,
    #[prost(message, optional, tag = "10")]
    pub pairing_request: Option<PairingRequest>,
    #[prost(message, optional, tag = "11")]
    pub pairing_request_ack: Option<PairingRequestAck>,
    #[prost(message, optional, tag = "20")]
    pub options: Option<PairingOptions>,
    #[prost(message, optional, tag = "30")]
    pub configuration: Option<PairingConfiguration>,
    #[prost(message, optional, tag = "31")]
    pub configuration_ack: Option<PairingConfigurationAck>,
    #[prost(message, optional, tag = "40")]
    pub secret: Option<PairingSecret>,
    #[prost(message, optional, tag = "41")]
    pub secret_ack: Option<PairingSecretAck>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingRequest {
    #[prost(string, tag = "1")]
    pub service_name: String,
    #[prost(string, tag = "2")]
    pub client_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingRequestAck {
    #[prost(string, tag = "1")]
    pub server_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingEncoding {
    #[prost(int32, tag = "1")]
    pub r#type: i32,
    #[prost(uint32, tag = "2")]
    pub symbol_length: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingOptions {
    #[prost(message, repeated, tag = "1")]
    pub input_encodings: Vec<PairingEncoding>,
    #[prost(message, repeated, tag = "2")]
    pub output_encodings: Vec<PairingEncoding>,
    #[prost(int32, tag = "3")]
    pub preferred_role: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingConfiguration {
    #[prost(message, optional, tag = "1")]
    pub encoding: Option<PairingEncoding>,
    #[prost(int32, tag = "2")]
    pub client_role: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingConfigurationAck {}

#[derive(Clone, PartialEq, Message)]
pub struct PairingSecret {
    #[prost(bytes = "vec", tag = "1")]
    pub secret: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PairingSecretAck {
    #[prost(bytes = "vec", tag = "1")]
    pub secret: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Remote protocol (port 6466)
// ---------------------------------------------------------------------------

/// Feature bitmask the official client advertises.
pub const REMOTE_FEATURES: i32 = 622;

pub const DIR_START_LONG: i32 = 1;
pub const DIR_END_LONG: i32 = 2;
pub const DIR_SHORT: i32 = 3;

#[derive(Clone, PartialEq, Message)]
pub struct RemoteMessage {
    #[prost(message, optional, tag = "1")]
    pub configure: Option<RemoteConfigure>,
    #[prost(message, optional, tag = "2")]
    pub set_active: Option<RemoteSetActive>,
    #[prost(message, optional, tag = "3")]
    pub error: Option<RemoteError>,
    #[prost(message, optional, tag = "8")]
    pub ping_request: Option<RemotePingRequest>,
    #[prost(message, optional, tag = "9")]
    pub ping_response: Option<RemotePingResponse>,
    #[prost(message, optional, tag = "10")]
    pub key_inject: Option<RemoteKeyInject>,
    #[prost(message, optional, tag = "20")]
    pub ime_key_inject: Option<RemoteImeKeyInject>,
    #[prost(message, optional, tag = "40")]
    pub start: Option<RemoteStart>,
    #[prost(message, optional, tag = "50")]
    pub set_volume_level: Option<RemoteSetVolumeLevel>,
    #[prost(message, optional, tag = "90")]
    pub app_link_launch: Option<RemoteAppLinkLaunchRequest>,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteDeviceInfo {
    #[prost(string, tag = "1")]
    pub model: String,
    #[prost(string, tag = "2")]
    pub vendor: String,
    #[prost(int32, tag = "3")]
    pub unknown1: i32,
    #[prost(string, tag = "4")]
    pub unknown2: String,
    #[prost(string, tag = "5")]
    pub package_name: String,
    #[prost(string, tag = "6")]
    pub app_version: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteConfigure {
    #[prost(int32, tag = "1")]
    pub code1: i32,
    #[prost(message, optional, tag = "2")]
    pub device_info: Option<RemoteDeviceInfo>,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteSetActive {
    #[prost(int32, tag = "1")]
    pub active: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteError {
    #[prost(bool, tag = "1")]
    pub value: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemotePingRequest {
    #[prost(int32, tag = "1")]
    pub val1: i32,
    #[prost(int32, tag = "2")]
    pub val2: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemotePingResponse {
    #[prost(int32, tag = "1")]
    pub val1: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteKeyInject {
    #[prost(int32, tag = "1")]
    pub key_code: i32,
    #[prost(int32, tag = "2")]
    pub direction: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteAppInfo {
    #[prost(string, tag = "10")]
    pub label: String,
    #[prost(string, tag = "12")]
    pub app_package: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteImeKeyInject {
    #[prost(message, optional, tag = "1")]
    pub app_info: Option<RemoteAppInfo>,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteStart {
    #[prost(bool, tag = "1")]
    pub started: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteSetVolumeLevel {
    #[prost(int32, tag = "6")]
    pub volume_max: i32,
    #[prost(int32, tag = "7")]
    pub volume_level: i32,
    #[prost(bool, tag = "8")]
    pub volume_muted: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct RemoteAppLinkLaunchRequest {
    #[prost(string, tag = "1")]
    pub app_link: String,
}
