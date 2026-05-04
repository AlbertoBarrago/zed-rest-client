/// Minimal JSONPath resolver supporting:
///   $.field
///   $.a.b.c
///   $.items[0]
///   $.items[0].name
///   $[0]
pub fn resolve(value: &serde_json::Value, path: &str) -> Option<String> {
    let inner = if path.starts_with("$.") {
        &path[2..]
    } else if path == "$" {
        return Some(to_string(value));
    } else if path.starts_with("$[") {
        &path[1..] // keep "[n]..." so the segment loop handles it
    } else {
        path // bare path without leading $
    };

    if inner.is_empty() {
        return Some(to_string(value));
    }

    let mut current = value;
    for segment in segments(inner) {
        current = step(current, &segment)?;
    }
    Some(to_string(current))
}

/// Split "a.b[0].c" into ["a", "b[0]", "c"].
fn segments(path: &str) -> Vec<String> {
    path.split('.').map(str::to_string).collect()
}

/// Navigate one path segment, handling "field", "field[n]", and "[n]".
fn step<'a>(value: &'a serde_json::Value, segment: &str) -> Option<&'a serde_json::Value> {
    if let Some(bracket) = segment.find('[') {
        let field = &segment[..bracket];
        let idx: usize = segment[bracket + 1..].trim_end_matches(']').parse().ok()?;
        let array = if field.is_empty() { value } else { value.get(field)? };
        array.get(idx)
    } else {
        value.get(segment)
    }
}

fn to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}
