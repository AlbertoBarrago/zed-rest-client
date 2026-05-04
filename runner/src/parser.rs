use std::collections::HashMap;

use crate::{cache, jsonpath};

#[derive(Debug, Clone)]
pub struct Request {
    pub name: Option<String>,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    /// 1-based line of the `###` separator that opens this section (or line 1 for the first section).
    pub section_line: usize,
}

pub fn parse(content: &str) -> Vec<Request> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    // Collect the start index of every section.
    // Section 0 always starts at line 0; each "###" line begins a new one.
    let mut section_starts: Vec<usize> = vec![0];
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim_start().starts_with("###") {
            section_starts.push(i);
        }
    }

    let mut requests = Vec::new();
    for (idx, &start) in section_starts.iter().enumerate() {
        let end = section_starts.get(idx + 1).copied().unwrap_or(lines.len());
        if let Some(req) = parse_section(&lines, start, end) {
            requests.push(req);
        }
    }
    requests
}

fn parse_section(lines: &[&str], start: usize, end: usize) -> Option<Request> {
    let mut name: Option<String> = None;
    let mut i = start;

    // A separator line (###) begins the section and may carry a name.
    if lines[start].trim_start().starts_with("###") {
        let after = lines[start].trim_start_matches('#').trim();
        if !after.is_empty() {
            name = Some(after.to_string());
        }
        i += 1;
    }

    // Skip blanks, comments and variable declarations; pick up @name if present.
    while i < end {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }
        if line.starts_with('#') || line.starts_with("//") {
            let rest = if line.starts_with('#') {
                line.trim_start_matches('#').trim()
            } else {
                line.trim_start_matches("//").trim()
            };
            if let Some(n) = rest.strip_prefix("@name").map(str::trim) {
                if !n.is_empty() {
                    name = Some(n.to_string());
                }
            }
            i += 1;
            continue;
        }
        if line.starts_with('@') {
            // File-level variable declaration (@var = value) — skip for now.
            i += 1;
            continue;
        }
        break;
    }

    if i >= end {
        return None;
    }

    let (method, url) = parse_request_line(lines[i])?;
    i += 1;

    // Headers: continue until a blank line or end of section.
    let mut headers: Vec<(String, String)> = Vec::new();
    while i < end {
        let line = lines[i];
        if line.trim().is_empty() {
            i += 1;
            break;
        }
        if let Some(colon) = line.find(':') {
            headers.push((
                line[..colon].trim().to_string(),
                line[colon + 1..].trim().to_string(),
            ));
        }
        i += 1;
    }

    // Body: everything that remains in the section, trimmed.
    let body = if i < end {
        let raw = lines[i..end].join("\n");
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    } else {
        None
    };

    Some(Request {
        name,
        method,
        url,
        headers,
        body,
        section_line: start + 1, // 1-based: the ### line (or line 1 for the implicit first section)
    })
}

fn parse_request_line(line: &str) -> Option<(String, String)> {
    const METHODS: &[&str] = &[
        "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD",
        "OPTIONS", "CONNECT", "TRACE", "LIST",
    ];
    let mut parts = line.splitn(3, ' ');
    let method = parts.next()?.trim().to_uppercase();
    if !METHODS.contains(&method.as_str()) {
        return None;
    }
    let url = parts.next()?.trim().to_string();
    if url.is_empty() {
        return None;
    }
    // Third part (HTTP/1.1) is intentionally ignored — reqwest handles protocol negotiation.
    Some((method, url))
}

/// Returns the request whose section contains `line` (1-based).
/// Matches from the `###` separator all the way to the next one, so the
/// cursor can be anywhere in the block — on the separator, comments, headers, or body.
pub fn find_at_line(requests: &[Request], line: usize) -> Option<&Request> {
    requests.iter().rev().find(|r| r.section_line <= line)
}

pub fn substitute_vars(mut req: Request, vars: &HashMap<String, String>) -> Request {
    req.url = substitute(&req.url, vars);
    req.headers = req
        .headers
        .iter()
        .map(|(k, v)| (k.clone(), substitute(v, vars)))
        .collect();
    req.body = req.body.map(|b| substitute(&b, vars));
    req
}

fn substitute(text: &str, vars: &HashMap<String, String>) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find("{{") {
        output.push_str(&remaining[..start]);
        let after_open = &remaining[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let expr = after_open[..end].trim();
            output.push_str(&resolve_var(expr, vars));
            remaining = &after_open[end + 2..];
        } else {
            output.push_str("{{");
            remaining = after_open;
        }
    }
    output.push_str(remaining);
    output
}

fn resolve_var(expr: &str, vars: &HashMap<String, String>) -> String {
    if let Some(rest) = expr.strip_prefix('$') {
        // Built-in variables.
        match rest {
            "guid" => return uuid::Uuid::new_v4().to_string(),
            "timestamp" => return std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
            "randomInt" => return (uuid::Uuid::new_v4().as_u128() % 1000).to_string(),
            _ if rest.starts_with("processEnv ") => {
                let key = rest["processEnv ".len()..].trim();
                return std::env::var(key).unwrap_or_default();
            }
            _ => {}
        }
    }

    // Response chaining: {{requestName.response.body.$.field}}
    //                    {{requestName.response.headers.Header-Name}}
    //                    {{requestName.response.status}}
    if let Some(resolved) = resolve_chained(expr) {
        return resolved;
    }

    // User-defined variable from env file.
    vars.get(expr)
        .cloned()
        .unwrap_or_else(|| format!("{{{{{}}}}}", expr))
}

/// Resolves `name.response.(status|headers.X|body.$.path)` from the local cache.
fn resolve_chained(expr: &str) -> Option<String> {
    let (req_name, rest) = expr.split_once(".response.")?;
    let cached = cache::load(req_name)?;

    if rest == "status" {
        return Some(cached.status.to_string());
    }
    if let Some(header_name) = rest.strip_prefix("headers.") {
        return cached
            .headers
            .get(&header_name.to_lowercase())
            .cloned();
    }
    if let Some(path) = rest.strip_prefix("body.") {
        let body: serde_json::Value = serde_json::from_str(&cached.body_raw).ok()?;
        return jsonpath::resolve(&body, path);
    }
    None
}
