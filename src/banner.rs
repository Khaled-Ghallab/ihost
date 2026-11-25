//! The startup banner: app name, where the folder is reachable, an optional
//! QR code for phones, and only the settings that differ from the defaults.

use std::{
    io::{self, IsTerminal, Write},
    net::IpAddr,
};

use crate::{
    config::{Config, DEFAULT_THREADS, detect_network_ip, local_url, network_url},
    handlers::UPLOAD_PAGE_QUERY,
    qr,
    util::{human_size, now_string},
};

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

const INDENT: &str = "  ";
const LABEL_WIDTH: usize = 10;

pub(crate) fn print(config: &Config) {
    let detected_network_ip = detect_network_ip(config.ip);
    let show_qr = io::stdout().is_terminal();
    let banner = render(config, detected_network_ip, show_qr, &now_string());

    // anstream drops the colors when output is redirected and translates them
    // on legacy Windows consoles.
    let _ = write!(anstream::stdout(), "{banner}");
}

fn render(
    config: &Config,
    detected_network_ip: Option<IpAddr>,
    show_qr: bool,
    started_at: &str,
) -> String {
    let mut out = String::new();
    let mut line = |text: String| {
        out.push_str(&text);
        out.push('\n');
    };

    line(String::new());
    line(format!(
        "{INDENT}{BOLD}iHost{RESET} {DIM}v{} · Powered by Khaled Ghallab{RESET}",
        env!("CARGO_PKG_VERSION")
    ));
    line(String::new());

    line(row("Folder", display_path(&config.root.to_string_lossy())));
    if let Some(url) = local_url(config) {
        line(row("Local", &format!("{CYAN}{url}{RESET}")));
    }
    let network_url = network_url(config, detected_network_ip);
    let network = match &network_url {
        Some(url) => format!("{BOLD}{CYAN}{url}{RESET}"),
        None if config.ip.is_loopback() => {
            format!("{DIM}add -H to share with your phone and other devices{RESET}")
        }
        None => format!("{DIM}not detected{RESET}"),
    };
    line(row("Network", &network));

    if let Some(drawing) = network_url
        .as_deref()
        .filter(|_| show_qr)
        .and_then(qr::render)
    {
        line(String::new());
        for qr_line in drawing.lines() {
            line(format!("{INDENT}{qr_line}"));
        }
    }

    let mut settings = changed_settings(config);
    // With --index the folder shows index.html, so point at the upload page.
    // It leads the settings group, right above the "Upload" row it belongs to.
    if config.upload.is_some() && config.index {
        let base = network_url.clone().or_else(|| local_url(config));
        if let Some(base) = base {
            settings.insert(
                0,
                (
                    "Upload at",
                    format!("{CYAN}{base}{}?{UPLOAD_PAGE_QUERY}{RESET}", config.base_url),
                ),
            );
        }
    }

    if !settings.is_empty() {
        line(String::new());
        for (label, value) in settings {
            line(row(label, &value));
        }
    }

    line(String::new());
    line(format!(
        "{INDENT}{DIM}Started {started_at} · press Ctrl+C to stop{RESET}"
    ));
    line(format!("{INDENT}{DIM}{}{RESET}", "─".repeat(48)));
    out
}

/// Drops the `\\?\` verbatim prefix Windows adds to canonicalized paths.
fn display_path(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

fn row(label: &str, value: &str) -> String {
    format!("{INDENT}{DIM}{label:<LABEL_WIDTH$}{RESET}{value}")
}

/// Settings that differ from the defaults, in the order a user cares about.
fn changed_settings(config: &Config) -> Vec<(&'static str, String)> {
    let mut rows = Vec::new();

    if let Some(upload) = &config.upload {
        rows.push((
            "Upload",
            format!(
                "on · max {} · CSRF token {}",
                human_size(config.upload_size_limit),
                upload.csrf_token
            ),
        ));
    }
    if let Some(auth) = &config.auth {
        rows.push(("Password", format!("required · user {}", auth.username)));
    }
    if let Some(cert) = &config.cert {
        let password = if config.certpass.is_some() {
            " · certificate password set"
        } else {
            ""
        };
        rows.push((
            "HTTPS",
            format!("on · {}{password}", cert.to_string_lossy()),
        ));
    }
    if config.index {
        rows.push(("Index", "index.html shown for folders".to_owned()));
    }
    if let Some(path) = &config.try_file_404 {
        rows.push(("Fallback", path.to_string_lossy().into_owned()));
    }
    if config.base_url != "/" {
        rows.push(("Base URL", config.base_url.clone()));
    }
    if let Some(location) = &config.redirect_to {
        rows.push(("Redirect", format!("all requests → {location}")));
    }

    let headers: Vec<&str> = [
        (config.cors, "CORS"),
        (config.coop, "COOP"),
        (config.coep, "COEP"),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    if !headers.is_empty() {
        rows.push(("Headers", headers.join(", ")));
    }

    if !config.compress.is_empty() {
        let patterns: Vec<String> = config
            .compress
            .iter()
            .map(|ext| format!("*{ext}"))
            .collect();
        rows.push(("Compress", patterns.join(", ")));
    }

    let disabled: Vec<&str> = [
        (config.cache, "cache"),
        (config.range, "range requests"),
        (config.sort, "sorting"),
    ]
    .into_iter()
    .filter_map(|(on, name)| (!on).then_some(name))
    .collect();
    if !disabled.is_empty() {
        rows.push(("Off", disabled.join(", ")));
    }

    if config.threads != DEFAULT_THREADS {
        rows.push(("Threads", config.threads.to_string()));
    }

    rows
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv4Addr, path::PathBuf};

    use super::*;
    use crate::config::{AuthConfig, UploadConfig, test_config};

    fn plain(text: &str) -> String {
        let mut out = String::new();
        let mut chars = text.chars();
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn lan() -> Option<IpAddr> {
        Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 23)))
    }

    #[test]
    fn default_banner_is_short() {
        let config = test_config(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let banner = plain(&render(&config, None, true, "2026-11-25 23:50:44"));
        let version = env!("CARGO_PKG_VERSION");

        assert_eq!(
            banner,
            format!(
                "\n  iHost v{version} · Powered by Khaled Ghallab\n\n  \
                 Folder    .\n  \
                 Local     http://127.0.0.1:8000\n  \
                 Network   add -H to share with your phone and other devices\n\n  \
                 Started 2026-11-25 23:50:44 · press Ctrl+C to stop\n  {}\n",
                "─".repeat(48)
            )
        );
    }

    #[test]
    fn app_name_is_the_first_line() {
        let config = test_config(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let banner = plain(&render(&config, None, false, "t"));
        let first = banner.lines().find(|line| !line.trim().is_empty()).unwrap();
        assert!(first.trim_start().starts_with("iHost v"));
        assert!(first.ends_with("Powered by Khaled Ghallab"));
    }

    #[test]
    fn exposed_banner_shows_network_url_and_qr_only_on_terminal() {
        let config = test_config(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let with_qr = plain(&render(&config, lan(), true, "t"));
        assert!(with_qr.contains("  Network   http://192.168.1.23:8000\n"));
        assert!(with_qr.contains('▀') || with_qr.contains('▄'));

        let without_qr = plain(&render(&config, lan(), false, "t"));
        assert!(!without_qr.contains('▀') && !without_qr.contains('▄'));

        let undetected = plain(&render(&config, None, true, "t"));
        assert!(undetected.contains("  Network   not detected\n"));
        assert!(!undetected.contains('▀'));
    }

    #[test]
    fn lists_only_changed_settings() {
        let mut config = test_config(IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert!(changed_settings(&config).is_empty());

        config.upload = Some(UploadConfig {
            csrf_token: "abc123".to_owned(),
        });
        config.auth = Some(AuthConfig {
            username: "me".to_owned(),
            password: "secret".to_owned(),
        });
        config.cert = Some(PathBuf::from("localhost.p12"));
        config.cors = true;
        config.coep = true;
        config.compress = vec![".js".to_owned(), ".css".to_owned()];
        config.cache = false;
        config.sort = false;
        config.threads = 8;

        let rows: Vec<String> = changed_settings(&config)
            .into_iter()
            .map(|(label, value)| format!("{label}: {value}"))
            .collect();
        assert_eq!(
            rows,
            [
                "Upload: on · max 8.0 MB · CSRF token abc123",
                "Password: required · user me",
                "HTTPS: on · localhost.p12",
                "Headers: CORS, COEP",
                "Compress: *.js, *.css",
                "Off: cache, sorting",
                "Threads: 8",
            ]
        );
    }

    #[test]
    fn strips_windows_verbatim_prefix_from_folder() {
        assert_eq!(display_path(r"\\?\D:\Photos"), r"D:\Photos");
        assert_eq!(display_path("/home/me/Photos"), "/home/me/Photos");
    }

    #[test]
    fn upload_page_link_only_with_index_and_upload() {
        let mut config = test_config(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        config.upload = Some(UploadConfig {
            csrf_token: "t".to_owned(),
        });
        assert!(!plain(&render(&config, lan(), false, "t")).contains("Upload at"));

        config.index = true;
        assert!(
            plain(&render(&config, lan(), false, "t"))
                .contains("  Upload at http://192.168.1.23:8000/?upload\n")
        );

        // Below the QR code there is a blank line, then the upload page link
        // directly above the "Upload" settings row.
        let with_qr = plain(&render(&config, lan(), true, "t"));
        let lines: Vec<&str> = with_qr.lines().collect();
        let at = lines
            .iter()
            .position(|line| line.starts_with("  Upload at "))
            .unwrap();
        assert_eq!(lines[at - 1], "");
        let qr_row = lines.iter().position(|line| line.contains('▀')).unwrap();
        assert!(qr_row < at - 1);
        assert!(lines[at + 1].starts_with("  Upload    on"));

        let mut local_only = config.clone();
        local_only.ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert!(
            plain(&render(&local_only, None, false, "t"))
                .contains("  Upload at http://127.0.0.1:8000/?upload\n")
        );
    }

    #[test]
    fn never_prints_the_password() {
        let mut config = test_config(IpAddr::V4(Ipv4Addr::LOCALHOST));
        config.auth = Some(AuthConfig {
            username: "me".to_owned(),
            password: "hunter2".to_owned(),
        });
        assert!(!render(&config, None, true, "t").contains("hunter2"));
    }
}
