use serde::{Deserialize, Deserializer};

pub fn normalize<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(normalize_string(&s))
}

pub fn normalize_string(s: &str) -> String { s.trim().to_lowercase() }
