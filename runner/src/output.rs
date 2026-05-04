use std::path::Path;

use colored::Colorize;

use crate::executor::Response;

pub fn print_response(response: &Response) {
    // Status line
    let status_str = format!("{} {}", response.status, response.status_text);
    let status_colored = match response.status {
        200..=299 => status_str.green().bold(),
        300..=399 => status_str.yellow().bold(),
        400..=499 => status_str.red().bold(),
        _ => status_str.red().bold(),
    };
    println!("{} {}", "HTTP/1.1".dimmed(), status_colored);
    println!();

    // Response headers
    for (name, value) in &response.headers {
        println!("{}: {}", name.cyan(), value);
    }
    println!();

    // Body
    println!("{}", pretty_body(&response.body, &response.headers));
    println!();

    // Summary bar
    let bar = "─".repeat(52);
    println!("{}", bar.dimmed());
    let summary_status = match response.status {
        200..=299 => status_str_short(response).green(),
        300..=399 => status_str_short(response).yellow(),
        _ => status_str_short(response).red(),
    };
    println!(
        "{}  {}  {}",
        summary_status,
        format!("{}ms", response.duration_ms).yellow(),
        format!("{} bytes", response.size_bytes).cyan(),
    );
}

/// Write the response to `path`. With `include_headers`, prepends the status
/// line and all headers before the body, mimicking curl's `-D` format.
pub fn save_to_file(response: &Response, path: &Path, include_headers: bool) {
    let content = if include_headers {
        let mut out = format!("HTTP/1.1 {} {}\n", response.status, response.status_text);
        for (name, value) in &response.headers {
            out.push_str(&format!("{}: {}\n", name, value));
        }
        out.push('\n');
        out.push_str(&pretty_body(&response.body, &response.headers));
        out
    } else {
        pretty_body(&response.body, &response.headers)
    };

    match std::fs::write(path, content) {
        Ok(_) => eprintln!("response saved to {}", path.display()),
        Err(e) => eprintln!("warning: could not save to {}: {}", path.display(), e),
    }
}

fn status_str_short(r: &Response) -> String {
    format!("{} {}", r.status, r.status_text)
}

fn pretty_body(body: &str, headers: &[(String, String)]) -> String {
    let content_type = headers
        .iter()
        .find(|(name, _)| name.to_lowercase() == "content-type")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");

    if content_type.contains("application/json") || content_type.contains("text/json") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
            return serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| body.to_string());
        }
    }
    body.to_string()
}
