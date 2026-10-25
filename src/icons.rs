//! Pixel-perfect vector icon painters for FryRemote.
//! Drawn using egui's painter for crisp rendering on any DPI without font dependencies.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(41, 121, 255);
pub const POWER_RED: Color32 = Color32::from_rgb(255, 77, 77);

// ---------------------------------------------------------------------------
// Top Mobile Status Bar (9:41, Signal, Wi-Fi, Battery)
// ---------------------------------------------------------------------------

pub fn draw_status_bar(ui: &mut Ui, width: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 24.0), egui::Sense::hover());
    let painter = ui.painter();
    let text_color = Color32::from_rgb(240, 244, 250);

    // Time: 9:41
    painter.text(
        pos2(rect.min.x + 12.0, rect.center().y),
        Align2::LEFT_CENTER,
        "9:41",
        FontId::proportional(13.0),
        text_color,
    );

    // Right-hand icons: Signal, Wi-Fi, Battery
    let right = rect.max.x - 12.0;
    let cy = rect.center().y;

    // Battery
    let batt_w = 21.0;
    let batt_h = 10.5;
    let batt_rect = Rect::from_min_size(pos2(right - batt_w, cy - batt_h * 0.5), vec2(batt_w, batt_h));
    painter.rect_stroke(batt_rect, 2.8, Stroke::new(1.2, text_color), StrokeKind::Inside);
    // Battery fill (~75%)
    let fill_rect = Rect::from_min_size(batt_rect.min + vec2(2.0, 2.0), vec2(12.5, batt_h - 4.0));
    painter.rect_filled(fill_rect, 1.5, text_color);
    // Battery terminal nipple
    painter.line_segment(
        [pos2(batt_rect.max.x + 1.2, cy - 2.5), pos2(batt_rect.max.x + 1.2, cy + 2.5)],
        Stroke::new(1.4, text_color),
    );

    // Wi-Fi icon
    let wifi_x = right - batt_w - 14.0;
    let wifi_c = pos2(wifi_x, cy + 3.0);
    painter.circle_filled(wifi_c, 1.3, text_color);
    // Arcs
    painter.add(Shape::line(
        vec![wifi_c + vec2(-4.0, -3.5), wifi_c + vec2(0.0, -6.0), wifi_c + vec2(4.0, -3.5)],
        Stroke::new(1.4, text_color),
    ));
    painter.add(Shape::line(
        vec![wifi_c + vec2(-7.5, -6.5), wifi_c + vec2(0.0, -9.5), wifi_c + vec2(7.5, -6.5)],
        Stroke::new(1.5, text_color),
    ));

    // Signal bars (4 bars)
    let sig_right = wifi_x - 14.0;
    let bar_w = 2.4;
    let bar_gap = 1.6;
    let heights = [3.0, 5.5, 8.0, 11.0];
    for (i, &h) in heights.iter().enumerate() {
        let x = sig_right - ((4 - i) as f32) * (bar_w + bar_gap);
        let bar_rect = Rect::from_min_size(pos2(x, cy + 4.5 - h), vec2(bar_w, h));
        painter.rect_filled(bar_rect, 0.8, text_color);
    }
}

// ---------------------------------------------------------------------------
// Header Brand Logo: Controller with Radiating Blue Wi-Fi Arcs
// ---------------------------------------------------------------------------

pub fn draw_brand_logo(painter: &egui::Painter, center: Pos2) {
    // 3 curved Wi-Fi waves above remote in vibrant electric blue
    let wave_c = center + vec2(0.0, -7.0);
    // Outer wave
    painter.add(Shape::line(
        vec![wave_c + vec2(-13.0, -5.5), wave_c + vec2(0.0, -11.5), wave_c + vec2(13.0, -5.5)],
        Stroke::new(2.4, ACCENT_BLUE),
    ));
    // Middle wave
    painter.add(Shape::line(
        vec![wave_c + vec2(-8.5, -2.5), wave_c + vec2(0.0, -7.0), wave_c + vec2(8.5, -2.5)],
        Stroke::new(2.2, ACCENT_BLUE),
    ));
    // Inner wave
    painter.add(Shape::line(
        vec![wave_c + vec2(-4.5, 0.5), wave_c + vec2(0.0, -2.8), wave_c + vec2(4.5, 0.5)],
        Stroke::new(2.0, ACCENT_BLUE),
    ));

    // Remote body
    let body_rect = Rect::from_center_size(center + vec2(0.0, 6.0), vec2(19.0, 24.0));
    painter.rect_filled(body_rect, 6.0, Color32::from_rgb(235, 242, 252));

    // D-Pad cross inside remote body
    let dpad_c = body_rect.center() + vec2(0.0, 2.0);
    let dpad_color = Color32::from_rgb(18, 30, 50);
    painter.rect_filled(Rect::from_center_size(dpad_c, vec2(3.0, 8.5)), 1.2, dpad_color);
    painter.rect_filled(Rect::from_center_size(dpad_c, vec2(8.5, 3.0)), 1.2, dpad_color);

    // Power dot at top of remote body
    painter.circle_filled(body_rect.center() + vec2(0.0, -6.5), 1.8, ACCENT_BLUE);
}

// ---------------------------------------------------------------------------
// TV Device Outline Icon
// ---------------------------------------------------------------------------

pub fn draw_tv(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Screen outline
    let screen = Rect::from_center_size(center + vec2(0.0, -2.5), vec2(26.0, 18.0));
    painter.rect_stroke(screen, 3.5, Stroke::new(1.8, color), StrokeKind::Inside);

    // Stand neck
    painter.line_segment(
        [center + vec2(0.0, 6.5), center + vec2(0.0, 10.0)],
        Stroke::new(2.0, color),
    );
    // Stand base plate
    painter.line_segment(
        [center + vec2(-7.0, 10.0), center + vec2(7.0, 10.0)],
        Stroke::new(2.0, color),
    );
}

// ---------------------------------------------------------------------------
// Chevrons
// ---------------------------------------------------------------------------

pub fn draw_chevron_down(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    painter.add(Shape::line(
        vec![center + vec2(-size, -size * 0.5), center + vec2(0.0, size * 0.5), center + vec2(size, -size * 0.5)],
        Stroke::new(2.0, color),
    ));
}

pub fn draw_chevron_up(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    painter.add(Shape::line(
        vec![center + vec2(-size, size * 0.5), center + vec2(0.0, -size * 0.5), center + vec2(size, size * 0.5)],
        Stroke::new(2.0, color),
    ));
}

// ---------------------------------------------------------------------------
// Settings Gear Icon
// ---------------------------------------------------------------------------

pub fn draw_gear(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let r = size * 0.7;
    painter.circle_stroke(center, r * 0.65, Stroke::new(1.8, color));

    // 8 teeth around perimeter
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI / 4.0;
        let p1 = center + vec2(angle.cos() * (r * 0.5), angle.sin() * (r * 0.5));
        let p2 = center + vec2(angle.cos() * (r * 1.05), angle.sin() * (r * 1.05));
        painter.line_segment([p1, p2], Stroke::new(2.2, color));
    }
}

// ---------------------------------------------------------------------------
// Power Icon (Red ⏻)
// ---------------------------------------------------------------------------

pub fn draw_power(painter: &egui::Painter, center: Pos2, size: f32, color: Color32) {
    let r = size;
    // Circle arc with opening at the top
    let count = 16;
    let pts: Vec<Pos2> = (0..=count)
        .map(|i| {
            let frac = i as f32 / count as f32;
            // 55 degrees to 305 degrees (leaving top ~100 degrees open)
            let rad = std::f32::consts::PI * 0.5 + (frac - 0.5) * (std::f32::consts::PI * 1.45);
            center + vec2(rad.cos() * r, rad.sin() * r)
        })
        .collect();
    painter.add(Shape::line(pts, Stroke::new(2.2, color)));

    // Vertical line through the top gap
    painter.line_segment(
        [center + vec2(0.0, -r * 1.25), center + vec2(0.0, -r * 0.1)],
        Stroke::new(2.2, color),
    );
}

// ---------------------------------------------------------------------------
// Speaker & Volume Icons
// ---------------------------------------------------------------------------

fn draw_speaker_cone(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Back box
    painter.rect_filled(
        Rect::from_min_max(center + vec2(-7.5, -3.0), center + vec2(-4.0, 3.0)),
        1.0,
        color,
    );
    // Front cone
    let cone = vec![
        center + vec2(-4.0, -3.0),
        center + vec2(-0.5, -6.5),
        center + vec2(-0.5, 6.5),
        center + vec2(-4.0, 3.0),
    ];
    painter.add(Shape::convex_polygon(cone, color, Stroke::NONE));
}

pub fn draw_speaker_card(painter: &egui::Painter, center: Pos2, color: Color32) {
    draw_speaker_cone(painter, center + vec2(-3.0, 0.0), color);
    // Sound wave arc
    painter.add(Shape::line(
        vec![center + vec2(0.0, -4.5), center + vec2(3.5, 0.0), center + vec2(0.0, 4.5)],
        Stroke::new(1.8, color),
    ));
}

pub fn draw_speaker_vol_down(painter: &egui::Painter, center: Pos2, color: Color32) {
    draw_speaker_cone(painter, center + vec2(-4.0, 0.0), color);
    // Minus symbol
    painter.line_segment(
        [center + vec2(2.0, 0.0), center + vec2(7.5, 0.0)],
        Stroke::new(2.0, color),
    );
}

pub fn draw_speaker_vol_up(painter: &egui::Painter, center: Pos2, color: Color32) {
    draw_speaker_cone(painter, center + vec2(-4.0, 0.0), color);
    // Plus symbol
    painter.line_segment(
        [center + vec2(2.0, 0.0), center + vec2(8.0, 0.0)],
        Stroke::new(2.0, color),
    );
    painter.line_segment(
        [center + vec2(5.0, -3.0), center + vec2(5.0, 3.0)],
        Stroke::new(2.0, color),
    );
}

pub fn draw_speaker_mute(painter: &egui::Painter, center: Pos2, color: Color32) {
    draw_speaker_cone(painter, center + vec2(-2.0, 0.0), color);
    // Diagonal slash
    painter.line_segment(
        [center + vec2(-6.5, 6.5), center + vec2(6.5, -6.5)],
        Stroke::new(2.2, Color32::from_rgb(255, 95, 95)),
    );
}

// ---------------------------------------------------------------------------
// TV Input Icon
// ---------------------------------------------------------------------------

pub fn draw_input(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Screen / Box on right
    let rect = Rect::from_center_size(center + vec2(2.5, 0.0), vec2(13.0, 12.0));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.6, color), StrokeKind::Inside);
    // Arrow pointing into box from left
    painter.line_segment(
        [center + vec2(-7.5, 0.0), center + vec2(0.5, 0.0)],
        Stroke::new(1.8, color),
    );
    painter.line_segment(
        [center + vec2(-2.5, -2.8), center + vec2(0.5, 0.0)],
        Stroke::new(1.8, color),
    );
    painter.line_segment(
        [center + vec2(-2.5, 2.8), center + vec2(0.5, 0.0)],
        Stroke::new(1.8, color),
    );
}

// ---------------------------------------------------------------------------
// Navigation Icons (Back, Home, Menu)
// ---------------------------------------------------------------------------

pub fn draw_back(painter: &egui::Painter, center: Pos2, color: Color32) {
    painter.line_segment(
        [center + vec2(5.5, 0.0), center + vec2(-5.5, 0.0)],
        Stroke::new(2.2, color),
    );
    painter.line_segment(
        [center + vec2(-1.5, -4.0), center + vec2(-5.5, 0.0)],
        Stroke::new(2.2, color),
    );
    painter.line_segment(
        [center + vec2(-1.5, 4.0), center + vec2(-5.5, 0.0)],
        Stroke::new(2.2, color),
    );
}

pub fn draw_home(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Roof
    let roof = vec![
        center + vec2(0.0, -6.5),
        center + vec2(-7.0, 0.0),
        center + vec2(7.0, 0.0),
    ];
    painter.add(Shape::convex_polygon(roof, color, Stroke::NONE));
    // House body
    painter.rect_filled(
        Rect::from_min_max(center + vec2(-5.2, 0.0), center + vec2(5.2, 6.5)),
        1.0,
        color,
    );
    // Doorway cutout
    painter.rect_filled(
        Rect::from_min_max(center + vec2(-1.8, 2.2), center + vec2(1.8, 6.5)),
        0.5,
        Color32::from_rgb(19, 31, 50),
    );
}

pub fn draw_menu(painter: &egui::Painter, center: Pos2, color: Color32) {
    let w = 6.5;
    for y in [-4.5, 0.0, 4.5] {
        painter.line_segment(
            [center + vec2(-w, y), center + vec2(w, y)],
            Stroke::new(2.0, color),
        );
    }
}

// ---------------------------------------------------------------------------
// Guide & Info Icons
// ---------------------------------------------------------------------------

pub fn draw_guide(painter: &egui::Painter, center: Pos2, color: Color32) {
    let rect = Rect::from_center_size(center, vec2(15.0, 12.0));
    painter.rect_stroke(rect, 2.0, Stroke::new(1.6, color), StrokeKind::Inside);
    // EPG grid lines
    painter.line_segment(
        [center + vec2(-7.5, -0.5), center + vec2(7.5, -0.5)],
        Stroke::new(1.2, color),
    );
    painter.line_segment(
        [center + vec2(-2.0, -6.0), center + vec2(-2.0, -0.5)],
        Stroke::new(1.2, color),
    );
    painter.line_segment(
        [center + vec2(2.0, -0.5), center + vec2(2.0, 6.0)],
        Stroke::new(1.2, color),
    );
}

pub fn draw_info(painter: &egui::Painter, center: Pos2, color: Color32) {
    painter.circle_stroke(center, 7.0, Stroke::new(1.6, color));
    // Dot
    painter.circle_filled(center + vec2(0.0, -3.2), 1.1, color);
    // Stem
    painter.line_segment(
        [center + vec2(0.0, -1.0), center + vec2(0.0, 3.8)],
        Stroke::new(1.8, color),
    );
}

// ---------------------------------------------------------------------------
// Media Icons (Prev, Play/Pause, Next)
// ---------------------------------------------------------------------------

pub fn draw_prev(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Two left-pointing triangles (<<)
    let tri1 = vec![
        center + vec2(1.0, 0.0),
        center + vec2(6.5, -4.8),
        center + vec2(6.5, 4.8),
    ];
    let tri2 = vec![
        center + vec2(-5.5, 0.0),
        center + vec2(0.0, -4.8),
        center + vec2(0.0, 4.8),
    ];
    painter.add(Shape::convex_polygon(tri1, color, Stroke::NONE));
    painter.add(Shape::convex_polygon(tri2, color, Stroke::NONE));
}

pub fn draw_play_pause(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Play triangle on left
    let play = vec![
        center + vec2(-6.5, -5.2),
        center + vec2(-6.5, 5.2),
        center + vec2(-1.0, 0.0),
    ];
    painter.add(Shape::convex_polygon(play, color, Stroke::NONE));

    // Two pause bars on right
    painter.line_segment(
        [center + vec2(2.8, -5.0), center + vec2(2.8, 5.0)],
        Stroke::new(2.2, color),
    );
    painter.line_segment(
        [center + vec2(6.5, -5.0), center + vec2(6.5, 5.0)],
        Stroke::new(2.2, color),
    );
}

pub fn draw_next(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Two right-pointing triangles (>>)
    let tri1 = vec![
        center + vec2(-1.0, 0.0),
        center + vec2(-6.5, -4.8),
        center + vec2(-6.5, 4.8),
    ];
    let tri2 = vec![
        center + vec2(5.5, 0.0),
        center + vec2(0.0, -4.8),
        center + vec2(0.0, 4.8),
    ];
    painter.add(Shape::convex_polygon(tri1, color, Stroke::NONE));
    painter.add(Shape::convex_polygon(tri2, color, Stroke::NONE));
}

// ---------------------------------------------------------------------------
// Search & Captions Icons
// ---------------------------------------------------------------------------

pub fn draw_search(painter: &egui::Painter, center: Pos2, color: Color32) {
    let circle_c = center + vec2(-1.8, -1.8);
    painter.circle_stroke(circle_c, 5.0, Stroke::new(1.8, color));
    painter.line_segment(
        [center + vec2(2.0, 2.0), center + vec2(6.5, 6.5)],
        Stroke::new(2.4, color),
    );
}

pub fn draw_captions(painter: &egui::Painter, center: Pos2, color: Color32) {
    let rect = Rect::from_center_size(center, vec2(17.0, 12.0));
    painter.rect_stroke(rect, 2.5, Stroke::new(1.6, color), StrokeKind::Inside);
    painter.text(
        center + vec2(0.0, -0.5),
        Align2::CENTER_CENTER,
        "CC",
        FontId::proportional(8.5),
        color,
    );
}

// ---------------------------------------------------------------------------
// D-Pad Arrow Triangles
// ---------------------------------------------------------------------------

pub fn draw_arrow_triangle(painter: &egui::Painter, center: Pos2, dir: Dir, size: f32, color: Color32) {
    let s = size;
    let pts = match dir {
        Dir::Up => vec![
            center + vec2(0.0, -s),
            center + vec2(-s * 1.25, s * 0.75),
            center + vec2(s * 1.25, s * 0.75),
        ],
        Dir::Down => vec![
            center + vec2(0.0, s),
            center + vec2(-s * 1.25, -s * 0.75),
            center + vec2(s * 1.25, -s * 0.75),
        ],
        Dir::Left => vec![
            center + vec2(-s, 0.0),
            center + vec2(s * 0.75, -s * 1.25),
            center + vec2(s * 0.75, s * 1.25),
        ],
        Dir::Right => vec![
            center + vec2(s, 0.0),
            center + vec2(-s * 0.75, -s * 1.25),
            center + vec2(-s * 0.75, s * 1.25),
        ],
    };
    painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}

// ---------------------------------------------------------------------------
// Streaming App Brand Logos
// ---------------------------------------------------------------------------

pub fn draw_youtube(painter: &egui::Painter, center: Pos2) {
    // Red pill
    let pill = Rect::from_center_size(center, vec2(21.0, 14.5));
    painter.rect_filled(pill, 4.5, Color32::from_rgb(255, 0, 0));
    // White right triangle inside
    let tri = vec![
        center + vec2(-2.5, -3.8),
        center + vec2(-2.5, 3.8),
        center + vec2(3.8, 0.0),
    ];
    painter.add(Shape::convex_polygon(tri, Color32::WHITE, Stroke::NONE));
}

pub fn draw_netflix(painter: &egui::Painter, center: Pos2) {
    let red_main = Color32::from_rgb(229, 9, 20);
    let red_bright = Color32::from_rgb(255, 30, 40);

    // Left vertical bar
    painter.line_segment(
        [center + vec2(-5.0, -7.0), center + vec2(-5.0, 7.0)],
        Stroke::new(2.8, red_main),
    );
    // Right vertical bar
    painter.line_segment(
        [center + vec2(5.0, -7.0), center + vec2(5.0, 7.0)],
        Stroke::new(2.8, red_main),
    );
    // Diagonal ribbon on top
    painter.line_segment(
        [center + vec2(-5.0, -7.0), center + vec2(5.0, 7.0)],
        Stroke::new(3.0, red_bright),
    );
}

pub fn draw_prime(painter: &egui::Painter, center: Pos2) {
    let prime_blue = Color32::from_rgb(0, 168, 225);
    // "prime" text
    painter.text(
        center + vec2(0.0, -2.5),
        Align2::CENTER_CENTER,
        "prime",
        FontId::proportional(11.0),
        prime_blue,
    );
    // Curved smile arrow
    painter.add(Shape::line(
        vec![center + vec2(-8.0, 4.0), center + vec2(0.0, 6.8), center + vec2(7.5, 4.0)],
        Stroke::new(1.6, prime_blue),
    ));
}

pub fn draw_disney(painter: &egui::Painter, center: Pos2) {
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "Disney+",
        FontId::proportional(11.5),
        Color32::WHITE,
    );
}

pub fn draw_hotstar(painter: &egui::Painter, center: Pos2) {
    // 5-pointed star outline
    let pts: Vec<Pos2> = (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { 7.5 } else { 3.4 };
            let a = (i as f32) * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2;
            center + vec2(a.cos() * r, a.sin() * r)
        })
        .collect();
    for i in 0..pts.len() {
        painter.line_segment([pts[i], pts[(i + 1) % pts.len()]], Stroke::new(1.6, Color32::WHITE));
    }
}

pub fn draw_spotify(painter: &egui::Painter, center: Pos2) {
    let spot_green = Color32::from_rgb(30, 215, 96);
    // Green circle
    painter.circle_filled(center, 8.0, spot_green);

    // 3 curved sound wave arcs in dark color
    let dark = Color32::from_rgb(18, 18, 18);
    painter.add(Shape::line(
        vec![center + vec2(-4.5, -2.6), center + vec2(0.0, -4.2), center + vec2(4.5, -2.6)],
        Stroke::new(1.4, dark),
    ));
    painter.add(Shape::line(
        vec![center + vec2(-4.0, 0.2), center + vec2(0.0, -1.2), center + vec2(4.0, 0.2)],
        Stroke::new(1.4, dark),
    ));
    painter.add(Shape::line(
        vec![center + vec2(-3.5, 2.8), center + vec2(0.0, 1.8), center + vec2(3.5, 2.8)],
        Stroke::new(1.4, dark),
    ));
}

// ---------------------------------------------------------------------------
// Bottom Navigation Bar Icons
// ---------------------------------------------------------------------------

pub fn draw_nav_remote(painter: &egui::Painter, center: Pos2, color: Color32) {
    let body = Rect::from_center_size(center, vec2(13.0, 19.0));
    painter.rect_stroke(body, 4.0, Stroke::new(1.6, color), StrokeKind::Inside);
    // Dpad cross
    painter.line_segment(
        [center + vec2(-2.8, -2.5), center + vec2(2.8, -2.5)],
        Stroke::new(1.4, color),
    );
    painter.line_segment(
        [center + vec2(0.0, -5.3), center + vec2(0.0, 0.3)],
        Stroke::new(1.4, color),
    );
    // Button dot
    painter.circle_filled(center + vec2(0.0, 4.5), 1.2, color);
}

pub fn draw_nav_apps(painter: &egui::Painter, center: Pos2, color: Color32) {
    // 2x2 grid of small rounded squares
    let sz = vec2(4.8, 4.8);
    painter.rect_filled(Rect::from_center_size(center + vec2(-3.8, -3.8), sz), 1.2, color);
    painter.rect_filled(Rect::from_center_size(center + vec2(3.8, -3.8), sz), 1.2, color);
    painter.rect_filled(Rect::from_center_size(center + vec2(-3.8, 3.8), sz), 1.2, color);
    painter.rect_filled(Rect::from_center_size(center + vec2(3.8, 3.8), sz), 1.2, color);
}

pub fn draw_nav_touchpad(painter: &egui::Painter, center: Pos2, color: Color32) {
    // Pointer arrow cursor ↖
    let pts = vec![
        center + vec2(-5.5, -6.5),
        center + vec2(-5.5, 5.0),
        center + vec2(-1.5, 1.8),
        center + vec2(2.8, 6.0),
        center + vec2(4.8, 4.2),
        center + vec2(0.5, 0.0),
        center + vec2(4.5, -1.0),
    ];
    painter.add(Shape::convex_polygon(pts, color, Stroke::NONE));
}

pub fn draw_nav_more(painter: &egui::Painter, center: Pos2, color: Color32) {
    // 3 horizontal dots
    painter.circle_filled(center + vec2(-5.5, 0.0), 1.8, color);
    painter.circle_filled(center + vec2(0.0, 0.0), 1.8, color);
    painter.circle_filled(center + vec2(5.5, 0.0), 1.8, color);
}
