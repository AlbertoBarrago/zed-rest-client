use std::collections::HashMap;
use std::path::Path;

pub fn load(env_file: &Path, env_name: Option<&str>) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = HashMap::new();

    let content = match std::fs::read_to_string(env_file) {
        Ok(c) => c,
        Err(_) => return vars,
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Warning: could not parse env file {}: {}", env_file.display(), e);
            return vars;
        }
    };

    let obj = match json.as_object() {
        Some(o) => o,
        None => return vars,
    };

    // Detect whether this is a flat { key: value } map or a nested { env: { key: value } } map.
    let is_nested = obj.values().all(|v| v.is_object());

    if is_nested {
        // Nested: pick the requested env, fall back to "local" / first available.
        let candidates: Vec<&str> = if let Some(n) = env_name {
            vec![n]
        } else {
            vec!["local", "development", "dev"]
        };

        let chosen = candidates
            .iter()
            .find_map(|name| obj.get(*name).and_then(|v| v.as_object()))
            .or_else(|| obj.values().next().and_then(|v| v.as_object()));

        if let Some(env) = chosen {
            for (k, v) in env {
                if let Some(s) = v.as_str() {
                    vars.insert(k.clone(), s.to_string());
                }
            }
        }
    } else {
        // Flat map: use all keys directly.
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                vars.insert(k.clone(), s.to_string());
            }
        }
    }

    vars
}
