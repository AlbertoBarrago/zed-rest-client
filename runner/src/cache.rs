use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CachedResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body_raw: String,
}

fn cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache").join("rest-runner")
}

fn cache_path(request_name: &str) -> PathBuf {
    cache_dir().join(format!("{}.json", sanitize(request_name)))
}

pub fn save(request_name: &str, response: &CachedResponse) {
    let dir = cache_dir();
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    if let Ok(json) = serde_json::to_string_pretty(response) {
        let _ = std::fs::write(cache_path(request_name), json);
    }
}

pub fn load(request_name: &str) -> Option<CachedResponse> {
    let content = std::fs::read_to_string(cache_path(request_name)).ok()?;
    serde_json::from_str(&content).ok()
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
