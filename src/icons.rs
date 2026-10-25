//! Vector icon painters and SVG asset renderers for FryRemote.
//! Follows clean modular structure with SVGs loaded from `assets/icons/`.

use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind, Ui};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub const ACCENT_BLUE: Color32 = Color32::from_rgb(59, 130, 246); // #3B82F6 Primary
pub const POWER_RED: Color32 = Color32::from_rgb(239, 68, 68);    // #EF4444 Crimson
pub const TEXT_SLATE: Color32 = Color32::from_rgb(100, 116, 139); // #64748B Slate

// ---------------------------------------------------------------------------
// Base SVG Icon Renderer using egui_extras
// ---------------------------------------------------------------------------

#[inline]
pub fn render_svg(ui: &Ui, rect: Rect, source: egui::ImageSource<'static>, tint: Option<Color32>) {
    let mut img = egui::Image::new(source);
    if let Some(c) = tint {
        img = img.tint(c);
    }
    img.paint_at(ui, rect);
}

// ---------------------------------------------------------------------------
// Mobile Top Status Bar (9:41, Signal, Wi-Fi, Battery, Camera Hole Punch)
// ---------------------------------------------------------------------------

pub fn draw_status_bar(ui: &mut Ui, width: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 22.0), egui::Sense::hover());
    let painter = ui.painter();
    let text_color = Color32::from_rgb(248, 250, 252); // #F8FAFC
    let cy = rect.center().y;

    // Time: 9:41
    painter.text(
        pos2(rect.min.x + 18.0, cy),
        Align2::LEFT_CENTER,
        "9:41",
        FontId::proportional(13.0),
        text_color,
    );

    // Center camera hole punch (matching modern phone bezel design)
    let cam_c = pos2(rect.center().x, cy);
    painter.circle_filled(cam_c, 5.0, Color32::from_rgb(7, 10, 18));
    painter.circle_stroke(cam_c, 5.0, Stroke::new(1.0, Color32::from_rgb(28, 38, 56)));

    // Right-hand icons: Signal, Wi-Fi, Battery
    let right = rect.max.x - 18.0;

    // Battery
    let batt_w = 21.0;
    let batt_h = 10.5;
    let batt_rect = Rect::from_min_size(pos2(right - batt_w, cy - batt_h * 0.5), vec2(batt_w, batt_h));
    painter.rect_stroke(batt_rect, 2.8, Stroke::new(1.2, text_color), StrokeKind::Inside);
    let fill_rect = Rect::from_min_size(batt_rect.min + vec2(2.0, 2.0), vec2(12.5, batt_h - 4.0));
    painter.rect_filled(fill_rect, 1.5, text_color);
    painter.line_segment(
        [pos2(batt_rect.max.x + 1.2, cy - 2.5), pos2(batt_rect.max.x + 1.2, cy + 2.5)],
        Stroke::new(1.4, text_color),
    );

    // Wi-Fi icon
    let wifi_x = right - batt_w - 14.0;
    let wifi_c = pos2(wifi_x, cy + 3.0);
    painter.circle_filled(wifi_c, 1.3, text_color);
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
// SVG Icon Helper Functions
// ---------------------------------------------------------------------------

pub fn svg_brand_logo(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/fryremote_logo.svg"), None);
}

pub fn svg_tv(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/tv.svg"), Some(color));
}

pub fn svg_power(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/power.svg"), None);
}

pub fn svg_volume_speaker(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/volume_speaker.svg"), Some(color));
}

pub fn svg_vol_up(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/volume_up.svg"), Some(color));
}

pub fn svg_vol_down(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/volume_down.svg"), Some(color));
}

pub fn svg_vol_mute(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/volume_mute.svg"), None);
}

pub fn svg_back(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/back.svg"), Some(color));
}

pub fn svg_home(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/home.svg"), Some(color));
}

pub fn svg_menu(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/menu.svg"), Some(color));
}

pub fn svg_input(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/input.svg"), Some(color));
}

pub fn svg_chevron_down(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/chevron_down.svg"), Some(color));
}

pub fn svg_chevron_up(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/chevron_up.svg"), Some(color));
}

pub fn svg_guide(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/guide.svg"), Some(color));
}

pub fn svg_info(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/info.svg"), Some(color));
}

pub fn svg_prev(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/prev.svg"), Some(color));
}

pub fn svg_play_pause(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/play_pause.svg"), Some(color));
}

pub fn svg_next(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/next.svg"), Some(color));
}

pub fn svg_search(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/search.svg"), Some(color));
}

pub fn svg_captions(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/captions.svg"), Some(color));
}

pub fn svg_settings(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/settings.svg"), Some(color));
}

pub fn svg_youtube(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/youtube.svg"), None);
}

pub fn svg_netflix(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/netflix.svg"), None);
}

pub fn svg_prime(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/prime_video.svg"), None);
}

pub fn svg_disney(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/disney_plus.svg"), None);
}

pub fn svg_hotstar(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/hotstar.svg"), None);
}

pub fn svg_spotify(ui: &Ui, rect: Rect) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/brand/spotify.svg"), None);
}

pub fn svg_nav_remote(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/nav/nav_remote.svg"), Some(color));
}

pub fn svg_nav_apps(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/nav/nav_apps.svg"), Some(color));
}

pub fn svg_nav_touchpad(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/nav/nav_touchpad.svg"), Some(color));
}

pub fn svg_nav_more(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/nav/nav_more.svg"), Some(color));
}

// ---------------------------------------------------------------------------
// Analytic Smooth Clover Geometry for Central D-Pad
// ---------------------------------------------------------------------------

pub fn get_clover_perimeter(center: Pos2, dist: f32, r_lobe: f32, r_fillet: f32) -> Vec<Pos2> {
    let dist_to_lobe = r_lobe + r_fillet;
    let discr = 2.0 * dist_to_lobe * dist_to_lobe - dist * dist;
    if discr <= 0.0 {
        return vec![];
    }
    let cf = (dist + discr.sqrt()) * std::f32::consts::FRAC_1_SQRT_2;

    let mut points = Vec::with_capacity(128);
    let n_lobe_pts = 16;
    let n_fillet_pts = 12;

    for k in 0..4 {
        let lobe_angle = (k as f32) * std::f32::consts::FRAC_PI_2;
        let c_lobe = center + vec2(dist * lobe_angle.cos(), dist * lobe_angle.sin());

        let f_next_rel = vec2(
            cf * (lobe_angle + std::f32::consts::FRAC_PI_4).cos() - dist * lobe_angle.cos(),
            cf * (lobe_angle + std::f32::consts::FRAC_PI_4).sin() - dist * lobe_angle.sin(),
        );
        let theta_end = f_next_rel.y.atan2(f_next_rel.x);

        let f_prev_rel = vec2(
            cf * (lobe_angle - std::f32::consts::FRAC_PI_4).cos() - dist * lobe_angle.cos(),
            cf * (lobe_angle - std::f32::consts::FRAC_PI_4).sin() - dist * lobe_angle.sin(),
        );
        let theta_start = f_prev_rel.y.atan2(f_prev_rel.x);

        let mut sweep = theta_end - theta_start;
        while sweep < 0.0 {
            sweep += std::f32::consts::TAU;
        }

        for i in 0..=n_lobe_pts {
            let frac = i as f32 / n_lobe_pts as f32;
            let a = theta_start + frac * sweep;
            points.push(c_lobe + vec2(r_lobe * a.cos(), r_lobe * a.sin()));
        }

        let f_angle = lobe_angle + std::f32::consts::FRAC_PI_4;
        let c_fillet = center + vec2(cf * f_angle.cos(), cf * f_angle.sin());
        let next_lobe_angle = lobe_angle + std::f32::consts::FRAC_PI_2;
        let c_next_lobe = center + vec2(dist * next_lobe_angle.cos(), dist * next_lobe_angle.sin());

        let a_start = (c_lobe.y - c_fillet.y).atan2(c_lobe.x - c_fillet.x);
        let a_end = (c_next_lobe.y - c_fillet.y).atan2(c_next_lobe.x - c_fillet.x);

        let mut f_sweep = a_end - a_start;
        while f_sweep > std::f32::consts::PI {
            f_sweep -= std::f32::consts::TAU;
        }
        while f_sweep < -std::f32::consts::PI {
            f_sweep += std::f32::consts::TAU;
        }

        for i in 1..n_fillet_pts {
            let frac = i as f32 / n_fillet_pts as f32;
            let a = a_start + frac * f_sweep;
            points.push(c_fillet + vec2(r_fillet * a.cos(), r_fillet * a.sin()));
        }
    }

    points
}

// ---------------------------------------------------------------------------
// Futuristic Clover D-Pad Painter with Neon Aura and SVG Arrow Icons
// ---------------------------------------------------------------------------

pub fn draw_clover_dpad(
    ui: &Ui,
    center: Pos2,
    dist: f32,
    r_lobe: f32,
    r_fillet: f32,
    hovered_dir: Option<Dir>,
    pressed_dir: Option<Dir>,
    center_hovered: bool,
    center_pressed: bool,
) {
    let points = get_clover_perimeter(center, dist, r_lobe, r_fillet);
    if points.len() < 3 {
        return;
    }

    let painter = ui.painter();

    // 1. Clover dark surface background mesh
    let bg_color = Color32::from_rgb(11, 19, 36); // #0B1324
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, bg_color);
    for &p in &points {
        mesh.colored_vertex(p, bg_color);
    }
    let n = points.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, i + 1, (i + 1) % n + 1);
    }
    painter.add(Shape::mesh(mesh));

    // 2. Electric neon blue glowing outline (multi-pass halo)
    let glow_outer = Color32::from_rgba_premultiplied(59, 130, 246, 25);
    let glow_mid = Color32::from_rgba_premultiplied(59, 130, 246, 70);
    let glow_inner = Color32::from_rgba_premultiplied(96, 165, 250, 140);
    let stroke_color = Color32::from_rgb(59, 130, 246); // #3B82F6

    painter.add(Shape::closed_line(points.clone(), Stroke::new(6.5, glow_outer)));
    painter.add(Shape::closed_line(points.clone(), Stroke::new(4.0, glow_mid)));
    painter.add(Shape::closed_line(points.clone(), Stroke::new(2.2, glow_inner)));
    painter.add(Shape::closed_line(points, Stroke::new(1.8, stroke_color)));

    // 3. Directional circular button pads
    let dirs = [
        (Dir::Up, vec2(0.0, -dist)),
        (Dir::Down, vec2(0.0, dist)),
        (Dir::Left, vec2(-dist, 0.0)),
        (Dir::Right, vec2(dist, 0.0)),
    ];

    let pad_radius = 21.0;
    for (dir, offset) in dirs {
        let lobe_c = center + offset;
        let is_pressed = pressed_dir == Some(dir);
        let is_hovered = hovered_dir == Some(dir);

        let pad_bg = if is_pressed {
            Color32::from_rgb(36, 56, 92)
        } else if is_hovered {
            Color32::from_rgb(27, 42, 68)
        } else {
            Color32::from_rgb(18, 28, 48)
        };

        ui.painter().circle_filled(lobe_c, pad_radius, pad_bg);
        ui.painter().circle_stroke(lobe_c, pad_radius, Stroke::new(1.0, Color32::from_rgb(28, 44, 72)));

        let arrow_color = if is_pressed {
            Color32::from_rgb(96, 165, 250)
        } else {
            Color32::from_rgb(248, 250, 252)
        };

        let arrow_rect = Rect::from_center_size(lobe_c, vec2(15.0, 15.0));
        match dir {
            Dir::Up => render_svg(ui, arrow_rect, egui::include_image!("../assets/icons/remote/arrow_up.svg"), Some(arrow_color)),
            Dir::Down => render_svg(ui, arrow_rect, egui::include_image!("../assets/icons/remote/arrow_down.svg"), Some(arrow_color)),
            Dir::Left => render_svg(ui, arrow_rect, egui::include_image!("../assets/icons/remote/arrow_left.svg"), Some(arrow_color)),
            Dir::Right => render_svg(ui, arrow_rect, egui::include_image!("../assets/icons/remote/arrow_right.svg"), Some(arrow_color)),
        }
    }

    // 4. Center OK button
    let center_r = 29.0;
    let ok_bg = if center_pressed {
        Color32::from_rgb(29, 78, 216) // #1D4ED8
    } else if center_hovered {
        Color32::from_rgb(96, 165, 250) // #60A5FA
    } else {
        Color32::from_rgb(59, 130, 246) // #3B82F6
    };

    ui.painter().circle_filled(center, center_r, ok_bg);
    ui.painter().circle_stroke(center, center_r, Stroke::new(1.5, Color32::from_rgb(147, 197, 253)));

    // TV icon from SVG inside center button
    let tv_rect = Rect::from_center_size(center + vec2(0.0, -7.5), vec2(18.0, 13.0));
    render_svg(ui, tv_rect, egui::include_image!("../assets/icons/remote/dpad_tv.svg"), Some(Color32::WHITE));

    ui.painter().text(
        center + vec2(0.0, 9.5),
        Align2::CENTER_CENTER,
        "OK",
        FontId::proportional(12.0),
        Color32::WHITE,
    );
}
