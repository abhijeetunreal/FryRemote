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

pub fn svg_mic(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/mic.svg"), Some(color));
}

pub fn svg_keyboard(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/keyboard.svg"), Some(color));
}

pub fn svg_tune(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/tune.svg"), Some(color));
}

pub fn svg_palette(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/palette.svg"), Some(color));
}

pub fn svg_replay_10(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/replay_10.svg"), Some(color));
}

pub fn svg_forward_10(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/forward_10.svg"), Some(color));
}

pub fn svg_movie(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/movie.svg"), Some(color));
}

pub fn svg_gamepad(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/gamepad.svg"), Some(color));
}

pub fn svg_touch_app(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/touch_app.svg"), Some(color));
}

pub fn svg_dialpad(ui: &Ui, rect: Rect, color: Color32) {
    render_svg(ui, rect, egui::include_image!("../assets/icons/remote/dialpad.svg"), Some(color));
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

    // 2. Electric neon blue glowing outline (multi-pass lush aura)
    let glow_wide = Color32::from_rgba_premultiplied(59, 130, 246, 20);
    let glow_outer = Color32::from_rgba_premultiplied(56, 189, 248, 45);
    let glow_mid = Color32::from_rgba_premultiplied(59, 130, 246, 95);
    let glow_inner = Color32::from_rgba_premultiplied(96, 165, 250, 160);
    let glow_core = Color32::from_rgba_premultiplied(191, 219, 254, 220);
    let stroke_color = Color32::from_rgb(56, 189, 248); // #38BDF8 bright neon cyan

    painter.add(Shape::closed_line(points.clone(), Stroke::new(10.0, glow_wide)));
    painter.add(Shape::closed_line(points.clone(), Stroke::new(6.5, glow_outer)));
    painter.add(Shape::closed_line(points.clone(), Stroke::new(4.2, glow_mid)));
    painter.add(Shape::closed_line(points.clone(), Stroke::new(2.6, glow_inner)));
    painter.add(Shape::closed_line(points.clone(), Stroke::new(1.6, glow_core)));
    painter.add(Shape::closed_line(points, Stroke::new(1.4, stroke_color)));

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

        let draw_r = if is_pressed { pad_radius - 1.5 } else { pad_radius };
        let stroke = if is_pressed {
            Stroke::new(2.0, Color32::from_rgb(56, 189, 248)) // Neon cyan press border
        } else {
            Stroke::new(1.0, Color32::from_rgb(30, 48, 78))
        };

        ui.painter().circle_filled(lobe_c, draw_r, pad_bg);
        ui.painter().circle_stroke(lobe_c, draw_r, stroke);

        let arrow_color = if is_pressed {
            Color32::from_rgb(96, 165, 250)
        } else {
            Color32::from_rgb(248, 250, 252)
        };

        let arrow_rect = Rect::from_center_size(lobe_c, vec2(17.0, 17.0));
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

    let draw_center_r = if center_pressed { center_r - 1.5 } else { center_r };
    let ok_stroke = if center_pressed {
        Stroke::new(2.5, Color32::from_rgb(224, 242, 254))
    } else {
        Stroke::new(1.8, Color32::from_rgb(147, 197, 253))
    };

    ui.painter().circle_filled(center, draw_center_r, ok_bg);
    ui.painter().circle_stroke(center, draw_center_r, ok_stroke);

    // TV icon from SVG inside center button
    let tv_rect = Rect::from_center_size(center + vec2(0.0, -8.0), vec2(21.0, 15.0));
    render_svg(ui, tv_rect, egui::include_image!("../assets/icons/remote/dpad_tv.svg"), Some(Color32::WHITE));

    ui.painter().text(
        center + vec2(0.0, 9.5),
        Align2::CENTER_CENTER,
        "OK",
        FontId::proportional(12.5),
        Color32::WHITE,
    );
}

// ---------------------------------------------------------------------------
// Material 3 Expressive Vector Icon Painters
// ---------------------------------------------------------------------------

pub fn draw_icon_mic(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r_pill = rect.height() * 0.22;
    let pill_top = c.y - rect.height() * 0.32;
    let pill_bot = c.y + rect.height() * 0.04;
    let pill_rect = Rect::from_x_y_ranges(c.x - r_pill..=c.x + r_pill, pill_top..=pill_bot);
    painter.rect_filled(pill_rect, r_pill, color);

    // Cradle arc
    let cradle_r = rect.width() * 0.35;
    let n = 12;
    let mut arc_pts = Vec::with_capacity(n);
    for i in 0..=n {
        let a = std::f32::consts::PI * (0.05 + 0.9 * (i as f32 / n as f32));
        arc_pts.push(pos2(c.x + cradle_r * a.cos(), c.y - rect.height() * 0.05 + cradle_r * a.sin()));
    }
    painter.add(Shape::line(arc_pts, Stroke::new(rect.width() * 0.09, color)));

    // Stem and base
    let stem_top = c.y - rect.height() * 0.05 + cradle_r;
    let stem_bot = rect.max.y - rect.height() * 0.12;
    painter.line_segment([pos2(c.x, stem_top), pos2(c.x, stem_bot)], Stroke::new(rect.width() * 0.09, color));
    painter.line_segment(
        [pos2(c.x - rect.width() * 0.22, stem_bot), pos2(c.x + rect.width() * 0.22, stem_bot)],
        Stroke::new(rect.width() * 0.09, color),
    );
}

pub fn draw_icon_keyboard(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width() * 0.86;
    let h = rect.height() * 0.60;
    let box_rect = Rect::from_center_size(rect.center(), vec2(w, h));
    painter.rect_stroke(box_rect, 3.5, Stroke::new(1.4, color), StrokeKind::Inside);

    let rows = 2;
    let cols = 4;
    for r in 0..rows {
        for c_idx in 0..cols {
            let x = box_rect.min.x + (c_idx as f32 + 0.5) * (w / cols as f32);
            let y = box_rect.min.y + (r as f32 + 0.45) * (h * 0.55 / rows as f32) + 2.0;
            painter.circle_filled(pos2(x, y), 1.0, color);
        }
    }
    let space_y = box_rect.max.y - 3.2;
    painter.line_segment([pos2(box_rect.min.x + w * 0.25, space_y), pos2(box_rect.max.x - w * 0.25, space_y)], Stroke::new(1.4, color));
}

pub fn draw_icon_palette(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.38;
    painter.circle_stroke(c, r, Stroke::new(1.4, color));
    let dots = [(-0.18, -0.15), (0.15, -0.18), (-0.12, 0.14), (0.16, 0.10)];
    for (dx, dy) in dots {
        painter.circle_filled(c + vec2(dx * rect.width(), dy * rect.height()), 1.4, color);
    }
}

pub fn draw_icon_tune(painter: &egui::Painter, rect: Rect, color: Color32) {
    let stroke = Stroke::new(1.4, color);
    let xs = [rect.min.x + rect.width() * 0.25, rect.center().x, rect.max.x - rect.width() * 0.25];
    let slider_ys = [0.35, 0.65, 0.40];

    for (i, &x) in xs.iter().enumerate() {
        painter.line_segment([pos2(x, rect.min.y + 3.0), pos2(x, rect.max.y - 3.0)], stroke);
        let knob_y = rect.min.y + rect.height() * slider_ys[i];
        painter.rect_filled(Rect::from_center_size(pos2(x, knob_y), vec2(6.0, 3.5)), 1.2, color);
    }
}

pub fn draw_icon_expand_more(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let w = rect.width() * 0.35;
    let h = rect.height() * 0.20;
    painter.add(Shape::line(
        vec![pos2(c.x - w, c.y - h * 0.5), pos2(c.x, c.y + h * 0.5), pos2(c.x + w, c.y - h * 0.5)],
        Stroke::new(1.8, color),
    ));
}

pub fn draw_icon_replay_10(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.36;
    let n = 16;
    let mut pts = Vec::with_capacity(n);
    for i in 0..=n {
        let a = std::f32::consts::PI * (0.3 + 1.5 * (i as f32 / n as f32));
        pts.push(pos2(c.x + r * a.cos(), c.y + r * a.sin()));
    }
    painter.add(Shape::line(pts, Stroke::new(1.5, color)));
    let start_p = pos2(c.x + r * (std::f32::consts::PI * 0.3).cos(), c.y + r * (std::f32::consts::PI * 0.3).sin());
    painter.add(Shape::convex_polygon(
        vec![start_p + vec2(-1.0, -4.0), start_p + vec2(4.0, 0.0), start_p + vec2(-4.0, 2.0)],
        color,
        Stroke::NONE,
    ));
    painter.text(c + vec2(0.0, 0.5), Align2::CENTER_CENTER, "10", FontId::proportional(8.5), color);
}

pub fn draw_icon_forward_10(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.36;
    let n = 16;
    let mut pts = Vec::with_capacity(n);
    for i in 0..=n {
        let a = std::f32::consts::PI * (-0.3 - 1.5 * (i as f32 / n as f32));
        pts.push(pos2(c.x + r * a.cos(), c.y + r * a.sin()));
    }
    painter.add(Shape::line(pts, Stroke::new(1.5, color)));
    let start_p = pos2(c.x + r * (-std::f32::consts::PI * 0.3).cos(), c.y + r * (-std::f32::consts::PI * 0.3).sin());
    painter.add(Shape::convex_polygon(
        vec![start_p + vec2(1.0, -4.0), start_p + vec2(-4.0, 0.0), start_p + vec2(4.0, 2.0)],
        color,
        Stroke::NONE,
    ));
    painter.text(c + vec2(0.0, 0.5), Align2::CENTER_CENTER, "10", FontId::proportional(8.5), color);
}

pub fn draw_icon_movie(painter: &egui::Painter, rect: Rect, color: Color32) {
    let box_rect = Rect::from_center_size(rect.center(), vec2(rect.width() * 0.78, rect.height() * 0.65));
    painter.rect_stroke(box_rect, 2.5, Stroke::new(1.4, color), StrokeKind::Inside);
    let sp_w = 2.0;
    let sp_h = 2.0;
    for i in 0..3 {
        let x = box_rect.min.x + 3.0 + (i as f32) * ((box_rect.width() - 8.0) / 2.0);
        painter.rect_filled(Rect::from_min_size(pos2(x, box_rect.min.y + 1.5), vec2(sp_w, sp_h)), 0.5, color);
        painter.rect_filled(Rect::from_min_size(pos2(x, box_rect.max.y - 3.5), vec2(sp_w, sp_h)), 0.5, color);
    }
    let c = box_rect.center();
    painter.add(Shape::convex_polygon(
        vec![c + vec2(-2.5, -3.5), c + vec2(3.5, 0.0), c + vec2(-2.5, 3.5)],
        color,
        Stroke::NONE,
    ));
}

pub fn draw_icon_gamepad(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let body = Rect::from_center_size(c, vec2(rect.width() * 0.8, rect.height() * 0.48));
    painter.rect_stroke(body, 5.0, Stroke::new(1.3, color), StrokeKind::Inside);
    painter.line_segment([pos2(body.min.x + 4.5, c.y - 2.5), pos2(body.min.x + 4.5, c.y + 2.5)], Stroke::new(1.2, color));
    painter.line_segment([pos2(body.min.x + 2.0, c.y), pos2(body.min.x + 7.0, c.y)], Stroke::new(1.2, color));
    painter.circle_filled(pos2(body.max.x - 4.0, c.y - 1.5), 1.0, color);
    painter.circle_filled(pos2(body.max.x - 6.5, c.y + 1.5), 1.0, color);
}

pub fn draw_icon_touch_app(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    painter.circle_stroke(c + vec2(0.0, -3.0), rect.width() * 0.22, Stroke::new(1.2, color));
    let finger_pts = vec![
        c + vec2(0.0, -3.0),
        c + vec2(0.0, 4.0),
        c + vec2(3.0, 6.0),
        c + vec2(5.0, 3.0),
        c + vec2(5.0, -1.0),
    ];
    painter.add(Shape::line(finger_pts, Stroke::new(1.2, color)));
}

pub fn draw_icon_dialpad(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = 1.6;
    let step = rect.width() * 0.22;
    for row in -1..=1 {
        for col in -1..=1 {
            let p = c + vec2((col as f32) * step, (row as f32) * step);
            painter.circle_filled(p, r, color);
        }
    }
}

pub fn draw_icon_check(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let pts = vec![c + vec2(-4.5, 0.0), c + vec2(-1.0, 3.5), c + vec2(5.0, -3.5)];
    painter.add(Shape::line(pts, Stroke::new(2.0, color)));
}

pub fn draw_icon_power_ring(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.35;
    let n = 20;
    let mut pts = Vec::with_capacity(n);
    for i in 0..=n {
        let a = std::f32::consts::PI * (0.68 + 1.64 * (i as f32 / n as f32));
        pts.push(pos2(c.x + r * a.cos(), c.y + r * a.sin()));
    }
    painter.add(Shape::line(pts, Stroke::new(2.0, color)));
    painter.line_segment([pos2(c.x, c.y - r - 1.0), pos2(c.x, c.y - 1.0)], Stroke::new(2.0, color));
}

pub fn draw_icon_return(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let pts = vec![
        c + vec2(4.5, -4.5),
        c + vec2(4.5, 1.5),
        c + vec2(-4.0, 1.5),
    ];
    painter.add(Shape::line(pts, Stroke::new(1.6, color)));
    painter.add(Shape::convex_polygon(
        vec![c + vec2(-4.0, -2.0), c + vec2(-6.5, 1.5), c + vec2(-4.0, 5.0)],
        color,
        Stroke::NONE,
    ));
}

pub fn draw_icon_closed_captions(painter: &egui::Painter, rect: Rect, color: Color32) {
    let box_rect = Rect::from_center_size(rect.center(), vec2(rect.width() * 0.8, rect.height() * 0.58));
    painter.rect_stroke(box_rect, 3.0, Stroke::new(1.3, color), StrokeKind::Inside);
    painter.text(box_rect.center(), Align2::CENTER_CENTER, "CC", FontId::proportional(rect.height() * 0.35), color);
}

pub fn draw_icon_speaker_group(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r1 = Rect::from_min_size(c + vec2(-rect.width() * 0.35, -rect.height() * 0.3), vec2(rect.width() * 0.42, rect.height() * 0.6));
    painter.rect_stroke(r1, 3.0, Stroke::new(1.2, color), StrokeKind::Inside);
    painter.circle_stroke(r1.center() + vec2(0.0, 2.0), 3.0, Stroke::new(1.0, color));
    let r2 = Rect::from_min_size(c + vec2(rect.width() * 0.05, -rect.height() * 0.15), vec2(rect.width() * 0.32, rect.height() * 0.45));
    painter.rect_stroke(r2, 2.5, Stroke::new(1.2, color), StrokeKind::Inside);
    painter.circle_stroke(r2.center() + vec2(0.0, 1.5), 2.2, Stroke::new(1.0, color));
}
