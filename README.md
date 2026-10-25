# TV Remote (Rust)

A remote control for **Google TV / Android TV**, written in Rust. It speaks the
same *Android TV Remote Service v2* protocol as Google's own remote app, so it
works with Google TV, Chromecast with Google TV, Sony/TCL/Xiaomi/Mi Box, Nvidia
Shield and other Android TV devices on the same Wi-Fi network.

## Features

- Automatic TV discovery (mDNS) or add by IP address
- One-time pairing with the 6-character code shown on the TV
- D-pad, OK, Back, Home, Menu, with press-and-hold repeat
- Swipe touchpad (swipe = arrows, tap = OK)
- Volume, mute, channel up/down, input source, guide, info, captions, search, settings
- Media keys: play/pause, stop, rewind, fast-forward, previous/next
- Power button
- Typing on the TV with a built-in keyboard (letters, digits, `. @ - /`, space, delete, enter)
- One-tap launch for YouTube, Netflix, Prime Video, Disney+, Hotstar, Spotify
- Live volume level and power state from the TV; auto-reconnect

Voice search is not available: Google does not expose it to third-party clients.

## Build the APK (no Android Studio needed)

1. Create a free GitHub repository and upload this whole folder (keep the
   `.github` folder).
2. Open the **Actions** tab, choose **Build APK**, press **Run workflow**
   (it also runs on every push).
3. After a few minutes download the **TVRemote-apk** artifact, unzip it and
   copy `TVRemote.apk` to your phone.
4. On the phone open the file and allow *Install unknown apps* when asked.

To be able to install future builds *over* the old one, add repository secrets
`KEYSTORE_B64` (base64 of your keystore) and `KEYSTORE_PASSWORD`. Without them a
new signing key is generated each build, so uninstall the old app first.

### Build locally instead

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-apk --locked
export ANDROID_HOME=$HOME/Android/Sdk                 # SDK with platform 34 + build-tools
export ANDROID_NDK_ROOT=$ANDROID_HOME/ndk/26.3.11579264
export CARGO_APK_RELEASE_KEYSTORE=$HOME/release.keystore
export CARGO_APK_RELEASE_KEYSTORE_PASSWORD=yourpassword
cargo apk build --release --lib
# APK: target/release/apk/TVRemote.apk
```

Requires Android 7.0 (API 24) or newer.

## Using it

1. Phone and TV on the same Wi-Fi.
2. Open the app; your TV appears under *On your network* (or use *Add TV by IP address*).
3. Tap **Add**, then **Pair**. A code appears on the TV; type it in the app.
4. From then on just tap **Open remote**.

## Try it on a computer

```sh
cargo run --release --bin tvremote-desktop
cargo test --release        # unit tests + end-to-end test against a mock TV
```

## How it works

| Piece | File |
| --- | --- |
| Protobuf messages (no `protoc` needed) | `src/proto.rs` |
| Varint-delimited framing | `src/wire.rs` |
| Client certificate (RSA-2048, self-signed) | `src/creds.rs` |
| TLS with client auth, TV certificate accepted | `src/tls.rs` |
| Pairing on port 6467 (secret = SHA-256 over both RSA keys + code) | `src/pairing.rs` |
| Remote session on port 6466 (keys, pings, app links, state) | `src/remote.rs` |
| mDNS `_androidtvremote2._tcp` | `src/discovery.rs` |
| UI (egui) | `src/app.rs` |
| Multicast lock for discovery on Android | `src/android.rs` |

The TV's certificate is accepted without a CA check because trust comes from the
pairing code, which is cryptographically bound to both certificates.
