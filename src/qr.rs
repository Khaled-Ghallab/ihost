use qrcode::{QrCode, render::unicode::Dense1x2};

/// Black text on a bright white background, so the code scans the same on
/// light and dark terminal themes (many phone scanners reject inverted codes).
const DARK_ON_LIGHT: &str = "\x1b[30;107m";
const RESET: &str = "\x1b[0m";

/// Draws `text` as a QR code using half-block characters (two modules per
/// character row), quiet zone included, each line wrapped in explicit colors.
pub(crate) fn render(text: &str) -> Option<String> {
    let code = QrCode::new(text.as_bytes()).ok()?;
    let drawing = code
        .render::<Dense1x2>()
        .dark_color(Dense1x2::Dark)
        .light_color(Dense1x2::Light)
        .quiet_zone(true)
        .build();

    let lines: Vec<String> = drawing
        .lines()
        .map(|line| format!("    {DARK_ON_LIGHT}{line}{RESET}"))
        .collect();
    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_colored_square_block() {
        let drawing = render("http://192.168.1.23:8000").unwrap();
        let lines: Vec<&str> = drawing.lines().collect();

        // Version 2 code: 25 modules + 2 * 4 quiet zone = 33 wide, 17 rows of half blocks.
        assert_eq!(lines.len(), 17);
        for line in &lines {
            let body = line
                .strip_prefix(&format!("    {DARK_ON_LIGHT}"))
                .and_then(|rest| rest.strip_suffix(RESET))
                .unwrap();
            assert_eq!(body.chars().count(), 33);
        }
    }

    #[test]
    fn rendering_is_deterministic() {
        assert_eq!(
            render("http://10.0.0.5:8000"),
            render("http://10.0.0.5:8000")
        );
    }
}
