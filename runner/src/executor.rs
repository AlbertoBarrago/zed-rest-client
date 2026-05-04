use std::time::{Duration, Instant};

use crate::parser::Request;

pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u64,
    pub size_bytes: usize,
}

pub fn execute(
    request: &Request,
    verbose: bool,
    timeout_secs: u64,
) -> Result<Response, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let method = reqwest::Method::from_bytes(request.method.as_bytes())?;
    let mut builder = client.request(method, &request.url);

    let mut has_content_type = false;
    for (name, value) in &request.headers {
        if name.to_lowercase() == "content-type" {
            has_content_type = true;
        }
        builder = builder.header(name.as_str(), value.as_str());
    }

    if let Some(body) = &request.body {
        // Auto-set Content-Type to application/json when the body looks like JSON.
        if !has_content_type && body.trim_start().starts_with('{') {
            builder = builder.header("Content-Type", "application/json");
        }
        builder = builder.body(body.clone());
    }

    if verbose {
        eprintln!("> {} {}", request.method, request.url);
        for (name, value) in &request.headers {
            eprintln!("> {}: {}", name, value);
        }
        if let Some(body) = &request.body {
            eprintln!(">");
            eprintln!("> {}", body);
        }
        eprintln!();
    }

    let start = Instant::now();
    let resp = builder.send()?;
    let duration_ms = start.elapsed().as_millis() as u64;

    let status = resp.status();
    let status_text = status.canonical_reason().unwrap_or("").to_string();
    let status_code = status.as_u16();

    let headers: Vec<(String, String)> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body = resp.text()?;
    let size_bytes = body.len();

    Ok(Response {
        status: status_code,
        status_text,
        headers,
        body,
        duration_ms,
        size_bytes,
    })
}
