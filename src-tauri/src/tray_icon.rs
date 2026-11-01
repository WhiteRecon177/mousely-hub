//! Pure drawing and text helpers for the tray icon (no Tauri here, so they can be unit tested).

pub const SIZE: u32 = 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BatteryView {
    /// No mouse found: empty grey battery.
    Missing,
    /// Level 0..=100, not charging.
    Level(u8),
    /// Level 0..=100 (None when unknown), charging: blue bolt.
    Charging(Option<u8>),
}

type Rgba = [u8; 4];

// "Ice" palette: pale sky-blue outline, pale cyan fill, soft red when low, periwinkle when charging.
const OUTLINE: Rgba = [156, 220, 245, 255];
const OUTLINE_LOW: Rgba = [148, 196, 232, 255];
const OUTLINE_CHARGING: Rgba = [141, 187, 234, 255];
const OUTLINE_MISSING: Rgba = [139, 163, 184, 255];
const CAVITY: Rgba = [14, 24, 36, 255];
const FILL: Rgba = [147, 224, 233, 255];
const FILL_LOW: Rgba = [219, 97, 124, 255];
const FILL_CHARGING: Rgba = [106, 142, 203, 255];
const BOLT: Rgba = [241, 249, 249, 255];
const BOLT_EDGE: Rgba = [10, 20, 40, 255];

// Bolt polygon, in icon pixels (fits inside the cavity).
const BOLT_POINTS: [(f32, f32); 7] = [
    (17.4, 8.6),
    (11.2, 16.4),
    (15.2, 16.4),
    (13.6, 23.4),
    (20.8, 14.6),
    (16.8, 14.6),
    (19.0, 8.6),
];

#[derive(Clone, Copy)]
enum Shape {
    Round { x0: f32, y0: f32, x1: f32, y1: f32, r: f32 },
    PolyShift { dx: f32, dy: f32 },
}

fn covers(shape: Shape, x: f32, y: f32) -> bool {
    match shape {
        Shape::PolyShift { dx, dy } => {
            let shifted: Vec<(f32, f32)> = BOLT_POINTS.iter().map(|(px, py)| (px + dx, py + dy)).collect();
            inside_polygon(x, y, &shifted)
        }
        Shape::Round { x0, y0, x1, y1, r } => {
            if x < x0 || x > x1 || y < y0 || y > y1 {
                return false;
            }

            // A narrow shape cannot have a corner radius bigger than half of its size.
            let r = r.min((x1 - x0) / 2.0).min((y1 - y0) / 2.0);

            // Distance to the nearest corner centre when inside a corner square.
            let cx = x.clamp(x0 + r, x1 - r);
            let cy = y.clamp(y0 + r, y1 - r);
            let (dx, dy) = (x - cx, y - cy);

            dx * dx + dy * dy <= r * r
        }
    }
}

fn inside_polygon(x: f32, y: f32, points: &[(f32, f32)]) -> bool {
    let mut inside = false;
    let mut previous = points.len() - 1;

    for current in 0..points.len() {
        let (xi, yi) = points[current];
        let (xj, yj) = points[previous];

        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }

        previous = current;
    }

    inside
}

/// Paints a shape with 4x4 samples per pixel (smooth edges), blending over what is already there.
fn paint(buffer: &mut [u8], shape: Shape, color: Rgba) {
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut hits = 0u32;

            for sy in 0..4 {
                for sx in 0..4 {
                    let px = x as f32 + (sx as f32 + 0.5) / 4.0;
                    let py = y as f32 + (sy as f32 + 0.5) / 4.0;

                    if covers(shape, px, py) {
                        hits += 1;
                    }
                }
            }

            if hits == 0 {
                continue;
            }

            let index = ((y * SIZE + x) * 4) as usize;
            let alpha = hits * u32::from(color[3]) / 16; // 0..=255
            let old_alpha = u32::from(buffer[index + 3]);
            let out_alpha = alpha + old_alpha * (255 - alpha) / 255;

            if out_alpha == 0 {
                continue;
            }

            for channel in 0..3 {
                let new = u32::from(color[channel]) * alpha;
                let old = u32::from(buffer[index + channel]) * old_alpha * (255 - alpha) / 255;
                buffer[index + channel] = ((new + old) / out_alpha).min(255) as u8;
            }

            buffer[index + 3] = out_alpha.min(255) as u8;
        }
    }
}

/// 32x32 RGBA image of a battery (horizontal), with a bolt when charging.
pub fn render(view: BatteryView) -> Vec<u8> {
    let mut buffer = vec![0u8; (SIZE * SIZE * 4) as usize];

    let (level, outline, fill) = match view {
        BatteryView::Missing => (None, OUTLINE_MISSING, FILL),
        BatteryView::Level(level) if level < 15 => (Some(level.min(100)), OUTLINE_LOW, FILL_LOW),
        BatteryView::Level(level) => (Some(level.min(100)), OUTLINE, FILL),
        BatteryView::Charging(level) => {
            (level.map(|value| value.min(100)), OUTLINE_CHARGING, FILL_CHARGING)
        }
    };

    // Body outline (a ring: outer shape, then the dark inside), and the small terminal.
    paint(&mut buffer, Shape::Round { x0: 1.2, y0: 6.7, x1: 27.8, y1: 25.3, r: 6.5 }, outline);
    paint(&mut buffer, Shape::Round { x0: 28.6, y0: 12.2, x1: 30.9, y1: 19.8, r: 1.15 }, outline);
    paint(&mut buffer, Shape::Round { x0: 3.6, y0: 9.1, x1: 25.4, y1: 22.9, r: 4.1 }, CAVITY);

    if let Some(level) = level {
        if level > 0 {
            let width = (21.0 * f32::from(level) / 100.0).max(3.5);
            paint(
                &mut buffer,
                Shape::Round { x0: 4.8, y0: 10.3, x1: 4.8 + width, y1: 21.7, r: 3.0 },
                fill,
            );
        }
    }

    if matches!(view, BatteryView::Charging(_)) {
        // Thin dark edge first (the polygon moved a little in 8 directions), then the white bolt.
        for (dx, dy) in [
            (-0.8, 0.0), (0.8, 0.0), (0.0, -0.8), (0.0, 0.8),
            (-0.6, -0.6), (0.6, -0.6), (-0.6, 0.6), (0.6, 0.6),
        ] {
            paint(&mut buffer, Shape::PolyShift { dx, dy }, BOLT_EDGE);
        }
        paint(&mut buffer, Shape::PolyShift { dx: 0.0, dy: 0.0 }, BOLT);
    }

    buffer
}

fn connection_label(connection: &str) -> &'static str {
    match connection {
        "wired" => "Wired",
        "wireless" => "Wireless",
        "bluetooth" => "Bluetooth",
        _ => "",
    }
}

/// Everything the tray needs, derived from one reading of the mouse.
pub struct TrayState {
    pub view: BatteryView,
    /// Menu line 1: the model name.
    pub title: String,
    /// Menu line 2, for example "49% - Wireless" (the charging bolt is shown in the icon).
    pub status: String,
    pub tooltip: String,
}

pub fn describe(
    connected: bool,
    name: &str,
    connection: &str,
    battery: Option<u32>,
    charging: bool,
    has_battery: bool,
) -> TrayState {
    if !connected {
        return TrayState {
            view: BatteryView::Missing,
            title: "No mouse detected".to_string(),
            status: String::new(),
            tooltip: "No mouse detected".to_string(),
        };
    }

    let level = battery.map(|value| value.min(100) as u8);
    let link = connection_label(connection);

    let view = if charging {
        BatteryView::Charging(level)
    } else if let (true, Some(level)) = (has_battery, level) {
        BatteryView::Level(level)
    } else {
        BatteryView::Missing
    };

    let mut parts: Vec<String> = Vec::new();

    if has_battery || charging {
        match (battery, charging) {
            (Some(value), true) => parts.push(format!("Charging {}%", value)),
            (Some(value), false) => parts.push(format!("Battery {}%", value)),
            (None, true) => parts.push("Charging".to_string()),
            (None, false) => {}
        }
    }

    if !link.is_empty() {
        parts.push(link.to_string());
    }

    let status = parts.join(" - ");
    let tooltip = if status.is_empty() {
        name.to_string()
    } else {
        format!("{}\n{}", name, status)
    };

    TrayState {
        view,
        title: name.to_string(),
        status,
        tooltip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
        let index = ((y * SIZE + x) * 4) as usize;
        [buffer[index], buffer[index + 1], buffer[index + 2], buffer[index + 3]]
    }

    #[test]
    fn image_has_the_right_size() {
        assert_eq!(render(BatteryView::Level(50)).len(), (SIZE * SIZE * 4) as usize);
    }

    #[test]
    fn full_battery_is_green_and_empty_one_is_not() {
        let full = render(BatteryView::Level(100));
        let empty = render(BatteryView::Level(0));
        assert_eq!(pixel(&full, 20, 16), FILL);
        assert_ne!(pixel(&empty, 8, 16), FILL);
    }

    #[test]
    fn low_battery_is_red() {
        let low = render(BatteryView::Level(10));
        assert_eq!(pixel(&low, 6, 16), FILL_LOW);
    }

    #[test]
    fn charging_draws_the_bolt() {
        let charging = render(BatteryView::Charging(Some(49)));
        assert_eq!(pixel(&charging, 16, 12), BOLT);
        let plain = render(BatteryView::Level(49));
        assert_ne!(pixel(&plain, 16, 12), BOLT);
    }

    #[test]
    fn text_for_a_charging_wired_mouse() {
        let state = describe(true, "PRO X", "wired", Some(49), true, true);
        assert_eq!(state.view, BatteryView::Charging(Some(49)));
        assert_eq!(state.status, "Charging 49% - Wired");
        assert_eq!(state.tooltip, "PRO X\nCharging 49% - Wired");
    }

    #[test]
    fn text_for_a_wireless_mouse() {
        let state = describe(true, "PRO X", "wireless", Some(80), false, true);
        assert_eq!(state.view, BatteryView::Level(80));
        assert_eq!(state.status, "Battery 80% - Wireless");
    }

    #[test]
    fn missing_mouse() {
        let state = describe(false, "", "unknown", None, false, false);
        assert_eq!(state.view, BatteryView::Missing);
        assert_eq!(state.tooltip, "No mouse detected");
    }

    #[test]
    fn mouse_without_battery_shows_only_the_name() {
        let state = describe(true, "Mouse", "wired", None, false, false);
        assert_eq!(state.view, BatteryView::Missing);
        assert_eq!(state.tooltip, "Mouse\nWired");
    }
}