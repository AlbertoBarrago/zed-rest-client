use std::collections::HashMap;

use crate::{cache, jsonpath};

#[derive(Debug, Clone)]
pub struct Request {
    pub name: Option<String>,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
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

pub fn parse_variables(content: &str) -> HashMap<String, String> {
    let mut variables = HashMap::new();
    let mut before_request = true;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("###") {
            before_request = true;
            continue;
        }

        if before_request && parse_request_line(trimmed).is_some() {
            before_request = false;
            continue;
        }

        if before_request {
            if let Some((name, value)) = parse_variable_declaration(trimmed) {
                variables.insert(name.to_string(), value.to_string());
            }
        }
    }

    variables
}

fn parse_variable_declaration(line: &str) -> Option<(&str, &str)> {
    let declaration = line.strip_prefix('@')?;
    let (name, value) = declaration.split_once('=')?;
    let name = name.trim();

    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_alphanumeric() || "_.$-".contains(character))
    {
        return None;
    }

    Some((name, value.trim()))
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
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    } else {
        None
    };

    Some(Request {
        name,
        method,
        url,
        headers,
        body,
    })
}

fn parse_request_line(line: &str) -> Option<(String, String)> {
    const METHODS: &[&str] = &[
        "GET",
        "POST",
        "PUT",
        "DELETE",
        "PATCH",
        "HEAD",
        "OPTIONS",
        "CONNECT",
        "TRACE",
        "LIST",
        "GRAPHQL",
        "WEBSOCKET",
    ];
    let line = line.trim();
    let method = line.split_whitespace().next()?.to_uppercase();
    if !METHODS.contains(&method.as_str()) {
        return None;
    }

    let mut url = line[method.len()..].trim_start();
    if let Some(version_start) = url.rfind(char::is_whitespace) {
        let possible_version = url[version_start..].trim();
        if is_http_version(possible_version) {
            url = url[..version_start].trim_end();
        }
    }

    if url.is_empty() {
        return None;
    }

    Some((method, url.to_string()))
}

fn is_http_version(value: &str) -> bool {
    value
        .strip_prefix("HTTP/")
        .is_some_and(|version| {
            !version.is_empty()
                && version
                    .chars()
                    .all(|character| character.is_ascii_digit() || character == '.')
        })
}

pub fn find_by_signature<'a>(
    requests: &'a [Request],
    method: &str,
    url: &str,
) -> Option<&'a Request> {
    let method = method.trim().to_uppercase();
    let url = url.trim();
    requests
        .iter()
        .find(|r| r.method == method && normalize_url(&r.url) == normalize_url(url))
}

fn normalize_url(url: &str) -> String {
    url.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(method: &str, url: &str) -> Request {
        Request {
            name: None,
            method: method.into(),
            url: url.into(),
            headers: vec![],
            body: None,
        }
    }

    #[test]
    fn find_by_method_and_url() {
        let reqs = vec![req("GET", "https://a.com"), req("POST", "https://b.com")];
        assert_eq!(
            find_by_signature(&reqs, "post", "https://b.com")
                .unwrap()
                .url,
            "https://b.com"
        );
    }

    #[test]
    fn find_by_signature_normalizes_wrapped_urls() {
        let reqs = vec![req("GET", "https://example.com/\n  users")];
        assert!(find_by_signature(&reqs, "GET", "https://example.com/users").is_some());
    }

    #[test]
    fn find_by_signature_returns_none_for_empty_list() {
        assert!(find_by_signature(&[], "GET", "https://x.com").is_none());
    }

    // ── parse ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_single_request_no_separator() {
        let src = "GET https://example.com\n";
        let reqs = parse(src);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].method, "GET");
        assert_eq!(reqs[0].url, "https://example.com");
    }

    #[test]
    fn parse_url_with_spaces() {
        let src = "GET {{baseUrl}}/laws?query=raccolta differenziata\n";
        let reqs = parse(src);

        assert_eq!(
            reqs[0].url,
            "{{baseUrl}}/laws?query=raccolta differenziata"
        );
        assert!(find_by_signature(
            &reqs,
            "GET",
            "{{baseUrl}}/laws?query=raccolta differenziata"
        )
        .is_some());
    }

    #[test]
    fn parse_request_line_ignores_http_version() {
        let reqs = parse("GET https://example.com/laws?query=waste collection HTTP/1.1\n");

        assert_eq!(
            reqs[0].url,
            "https://example.com/laws?query=waste collection"
        );
    }

    #[test]
    fn parse_two_named_sections() {
        // 0: ### First
        // 1: GET https://a.com
        // 2: (blank)
        // 3: ### Second
        // 4: POST https://b.com
        let src = "### First\nGET https://a.com\n\n### Second\nPOST https://b.com\n";
        let reqs = parse(src);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].name.as_deref(), Some("First"));
        assert_eq!(reqs[1].name.as_deref(), Some("Second"));
    }

    #[test]
    fn parse_empty_separator_is_skipped() {
        // A bare `###` with no method should not produce a request.
        let src = "GET https://a.com\n\n###\n\n### Real\nDELETE https://b.com\n";
        let reqs = parse(src);
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[1].method, "DELETE");
    }

    #[test]
    fn parse_name_from_at_name_comment() {
        let src = "### \n# @name myReq\nGET https://x.com\n";
        let reqs = parse(src);
        assert_eq!(reqs[0].name.as_deref(), Some("myReq"));
    }

    #[test]
    fn parse_headers_and_body() {
        let src = "### T\nPOST https://x.com\nContent-Type: application/json\n\n{\"k\":1}\n";
        let reqs = parse(src);
        assert_eq!(
            reqs[0].headers,
            vec![("Content-Type".into(), "application/json".into())]
        );
        assert_eq!(reqs[0].body.as_deref(), Some("{\"k\":1}"));
    }

    #[test]
    fn parse_graphql_and_websocket_methods() {
        let src =
            "GRAPHQL https://api.example.com/graphql\n\n### WS\nWEBSOCKET ws://localhost:3000\n";
        let reqs = parse(src);
        assert_eq!(reqs[0].method, "GRAPHQL");
        assert_eq!(reqs[1].method, "WEBSOCKET");
    }

    #[test]
    fn parse_file_variables() {
        let src = "@baseUrl = http://localhost:8000/api/v1\n@token = prefix=value\n\nGET {{baseUrl}}/health\n";
        let variables = parse_variables(src);

        assert_eq!(
            variables.get("baseUrl").map(String::as_str),
            Some("http://localhost:8000/api/v1")
        );
        assert_eq!(
            variables.get("token").map(String::as_str),
            Some("prefix=value")
        );
    }

    #[test]
    fn parse_variables_ignores_request_body() {
        let src = "POST https://example.com\n\n@body = not-a-variable\n";

        assert!(parse_variables(src).is_empty());
    }

    #[test]
    fn substitute_file_variables_in_request() {
        let request = Request {
            name: None,
            method: "POST".into(),
            url: "{{baseUrl}}/health".into(),
            headers: vec![("Authorization".into(), "Bearer {{token}}".into())],
            body: Some("{\"url\":\"{{baseUrl}}\"}".into()),
        };
        let variables = HashMap::from([
            ("baseUrl".into(), "http://localhost:8000/api/v1".into()),
            ("token".into(), "secret".into()),
        ]);

        let request = substitute_vars(request, &variables);

        assert_eq!(request.url, "http://localhost:8000/api/v1/health");
        assert_eq!(request.headers[0].1, "Bearer secret");
        assert_eq!(
            request.body.as_deref(),
            Some("{\"url\":\"http://localhost:8000/api/v1\"}")
        );
    }

    #[test]
    fn substitute_nested_file_variables() {
        let request = req("GET", "https://example.com");
        let request = Request {
            headers: vec![("Authorization".into(), "Bearer {{token}}".into())],
            ..request
        };
        let variables = HashMap::from([
            ("token".into(), "{{loginToken}}".into()),
            ("loginToken".into(), "secret".into()),
        ]);

        let request = substitute_vars(request, &variables);

        assert_eq!(request.headers[0].1, "Bearer secret");
    }

    #[test]
    fn cyclic_variables_remain_unresolved() {
        let request = req("GET", "https://example.com/{{a}}");
        let variables = HashMap::from([
            ("a".into(), "{{b}}".into()),
            ("b".into(), "{{a}}".into()),
        ]);

        let request = substitute_vars(request, &variables);

        assert_eq!(request.url, "https://example.com/{{a}}");
    }
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
    substitute_nested(text, vars, &mut Vec::new())
}

fn substitute_nested(
    text: &str,
    vars: &HashMap<String, String>,
    resolving: &mut Vec<String>,
) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find("{{") {
        output.push_str(&remaining[..start]);
        let after_open = &remaining[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let expr = after_open[..end].trim();
            output.push_str(&resolve_var(expr, vars, resolving));
            remaining = &after_open[end + 2..];
        } else {
            output.push_str("{{");
            remaining = after_open;
        }
    }
    output.push_str(remaining);
    output
}

fn resolve_var(
    expr: &str,
    vars: &HashMap<String, String>,
    resolving: &mut Vec<String>,
) -> String {
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

    // User-defined variable from the request or env file. Values may themselves
    // contain variables, for example @token = {{login.response.body.access_token}}.
    let Some(value) = vars.get(expr) else {
        return format!("{{{{{}}}}}", expr);
    };

    const MAX_NESTING_DEPTH: usize = 32;
    if resolving.len() >= MAX_NESTING_DEPTH || resolving.iter().any(|name| name == expr) {
        return format!("{{{{{}}}}}", expr);
    }

    resolving.push(expr.to_string());
    let resolved = substitute_nested(value, vars, resolving);
    resolving.pop();
    resolved
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
