use std::path::Path;

use colored::Colorize;

use crate::executor::Response;
use crate::parser::Request;

pub fn print_request_header(request: &Request) {
    let method_colored = match request.method.as_str() {
        "GET"    => request.method.green().bold(),
        "POST"   => request.method.yellow().bold(),
        "PUT"    => request.method.blue().bold(),
        "PATCH"  => request.method.cyan().bold(),
        "DELETE" => request.method.red().bold(),
        _        => request.method.white().bold(),
    };

    println!("{}", "REST Client".bright_white().bold());
    if let Some(name) = &request.name {
        println!("{} {}", "Request".dimmed(), name.bold());
        println!("{} {} {}", "Target ".dimmed(), method_colored, request.url);
    } else {
        println!("{} {} {}", "Target ".dimmed(), method_colored, request.url);
    }
}

pub fn print_response(response: &Response, show_headers: bool) {
    let status_str = format!("{} {}", response.status, response.status_text);
    let status_colored = match response.status {
        200..=299 => status_str.green().bold(),
        300..=399 => status_str.yellow().bold(),
        400..=499 => status_str.red().bold(),
        _ => status_str.red().bold(),
    };

    println!(
        "{} {}  {}  {}  {}",
        "Status ".dimmed(),
        status_colored,
        format_duration(response.duration_ms).yellow(),
        format_size(response.size_bytes).cyan(),
        content_type(response.headers.as_slice()).dimmed(),
    );
    println!("{}", "─".repeat(80).dimmed());

    if show_headers {
        println!("{}", "Headers".bright_white().bold());
        for (name, value) in &response.headers {
            println!("{}: {}", name.cyan(), value);
        }
        println!("{}", "─".repeat(80).dimmed());
    }

    println!("{}", "Body".bright_white().bold());
    print!("{}", body_preview(&response.body, &response.headers));
    println!();

    if !show_headers {
        println!(
            "{}",
            "Tip: pass --headers to show response headers.".dimmed()
        );
    }
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

fn body_preview(body: &str, headers: &[(String, String)]) -> String {
    let pretty = pretty_body(body, headers);
    if pretty.ends_with('\n') {
        pretty
    } else {
        format!("{}\n", pretty)
    }
}

fn content_type(headers: &[(String, String)]) -> &str {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.as_str())
        .unwrap_or("unknown content-type")
}

fn format_duration(ms: u64) -> String {
    if ms < 1_000 {
        format!("{}ms", ms)
    } else {
        format!("{:.2}s", ms as f64 / 1_000.0)
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1_024 {
        format!("{} B", bytes)
    } else if bytes < 1_024 * 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1_024.0 * 1_024.0))
    }
}
