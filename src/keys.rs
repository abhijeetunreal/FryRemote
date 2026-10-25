//! Android `KeyEvent` key codes used by the remote.

pub const HOME: i32 = 3;
pub const BACK: i32 = 4;
pub const DPAD_UP: i32 = 19;
pub const DPAD_DOWN: i32 = 20;
pub const DPAD_LEFT: i32 = 21;
pub const DPAD_RIGHT: i32 = 22;
pub const DPAD_CENTER: i32 = 23;
pub const VOLUME_UP: i32 = 24;
pub const VOLUME_DOWN: i32 = 25;
pub const POWER: i32 = 26;
pub const SPACE: i32 = 62;
pub const ENTER: i32 = 66;
pub const DEL: i32 = 67;
pub const SEARCH: i32 = 84;
pub const MENU: i32 = 82;
pub const MEDIA_PLAY_PAUSE: i32 = 85;
pub const MEDIA_STOP: i32 = 86;
pub const MEDIA_NEXT: i32 = 87;
pub const MEDIA_PREVIOUS: i32 = 88;
pub const MEDIA_REWIND: i32 = 89;
pub const MEDIA_FAST_FORWARD: i32 = 90;
pub const VOLUME_MUTE: i32 = 164;
pub const INFO: i32 = 165;
pub const CHANNEL_UP: i32 = 166;
pub const CHANNEL_DOWN: i32 = 167;
pub const TV: i32 = 170;
pub const GUIDE: i32 = 172;
pub const CAPTIONS: i32 = 175;
pub const SETTINGS: i32 = 176;
pub const TV_INPUT: i32 = 178;
pub const PERIOD: i32 = 56;
pub const COMMA: i32 = 55;
pub const MINUS: i32 = 69;
pub const AT: i32 = 77;
pub const SLASH: i32 = 76;

/// Map a typed character to an Android key code. Letters are case-insensitive
/// (the TV applies its own shift state).
pub fn key_for_char(c: char) -> Option<i32> {
    match c {
        'a'..='z' => Some(29 + (c as i32 - 'a' as i32)),
        'A'..='Z' => Some(29 + (c as i32 - 'A' as i32)),
        '0'..='9' => Some(7 + (c as i32 - '0' as i32)),
        ' ' => Some(SPACE),
        '.' => Some(PERIOD),
        ',' => Some(COMMA),
        '-' => Some(MINUS),
        '@' => Some(AT),
        '/' => Some(SLASH),
        '\n' => Some(ENTER),
        _ => None,
    }
}

/// Deep links for quick-launch buttons.
pub const APPS: &[(&str, &str)] = &[
    ("YouTube", "https://www.youtube.com"),
    ("Netflix", "https://www.netflix.com/title"),
    ("Prime", "https://app.primevideo.com"),
    ("Disney+", "https://www.disneyplus.com"),
    ("Hotstar", "https://www.hotstar.com"),
    ("Spotify", "spotify://"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_and_digits() {
        assert_eq!(key_for_char('a'), Some(29));
        assert_eq!(key_for_char('Z'), Some(54));
        assert_eq!(key_for_char('0'), Some(7));
        assert_eq!(key_for_char('9'), Some(16));
        assert_eq!(key_for_char('€'), None);
    }
}
