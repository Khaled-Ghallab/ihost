use qrcode::{QrCode, render::unicode::Dense1x2};

/// Black text on a bright white background, so the code scans the same on
/// light and dark terminal themes (many phone scanners reject inverted codes).
const DARK_ON_LIGHT: &str = "\x1b[30;107m";
const RESET: &str = "\x1b[0m";

/// Light border around the code, in modules. The spec asks for 4 (meant for
/// print); phone cameras read a 2-module border on screen reliably, and it
/// keeps the code compact in the terminal.
const QUIET_ZONE: usize = 2;

/// Draws `text` as a QR code using half-block characters (two modules per
/// character row) with a slim quiet zone, each line wrapped in explicit colors.
pub(crate) fn render(text: &str) -> Option<String> {
    let code = QrCode::new(text.as_bytes()).ok()?;
    let drawing = code
        .render::<Dense1x2>()
        .dark_color(Dense1x2::Dark)
        .light_color(Dense1x2::Light)
        .quiet_zone(false)
        .build();

    let side = " ".repeat(QUIET_ZONE);
    let body: Vec<String> = drawing
        .lines()
        .map(|line| format!("{side}{line}{side}"))
        .collect();
    let width = body.first()?.chars().count();
    // One character row covers two modules vertically.
    let blank = vec![" ".repeat(width); QUIET_ZONE / 2];

    let lines: Vec<String> = blank
        .iter()
        .chain(&body)
        .chain(&blank)
        .map(|line| format!("    {DARK_ON_LIGHT}{line}{RESET}"))
        .collect();
    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_colored_square_block_with_slim_border() {
        let drawing = render("http://192.168.1.23:8000").unwrap();
        let lines: Vec<&str> = drawing.lines().collect();

        // Version 2 code: 25 modules + 2 * 2 quiet zone = 29 wide;
        // 13 half-block rows for 25 modules, plus one blank row above and below.
        assert_eq!(lines.len(), 15);
        for line in &lines {
            let body = line
                .strip_prefix(&format!("    {DARK_ON_LIGHT}"))
                .and_then(|rest| rest.strip_suffix(RESET))
                .unwrap();
            assert_eq!(body.chars().count(), 29);
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
