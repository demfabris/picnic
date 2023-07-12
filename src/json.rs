use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::output::SEPARATOR_THAT_WONT_COLIDE_FOR_SURE;
use crate::Result;

pub type Map = BTreeMap<String, Value>;

// Fixes json data like: { "foo": $bar, "baz": { "borg": $boo } }
pub fn into_fixed(value: &str) -> Result<Value> {
    let re = Regex::new(r"\$\s*\b(\w+)\b").unwrap();
    let corrected_str =
        re.replace_all(value, |caps: &regex::Captures| format!(r#""{}""#, &caps[1]));
    Ok(serde_json::from_str(&corrected_str)?)
}

pub fn dfs_flatten(data: &Value, flattened: &mut BTreeMap<String, Value>, parent: &str) {
    match data {
        _map if data.is_object() => {
            if let Some(map) = data.as_object() {
                for (key, value) in map {
                    if value.is_object() || value.is_array() {
                        dfs_flatten(value, flattened, key.as_str());
                    } else {
                        if parent.is_empty() {
                            flattened.insert(key.clone(), value.clone());
                            continue;
                        }
                        flattened.insert(
                            format!(
                                "{}{}{}",
                                parent,
                                SEPARATOR_THAT_WONT_COLIDE_FOR_SURE,
                                key.as_str()
                            ),
                            value.clone(),
                        );
                    }
                }
            }
        }
        _array if data.is_array() => {
            if let Some(array) = data.as_array() {
                for (idx, value) in array.iter().enumerate() {
                    dfs_flatten(
                        value,
                        flattened,
                        format!("{parent}{SEPARATOR_THAT_WONT_COLIDE_FOR_SURE}{idx}").as_str(),
                    );
                }
            }
        }
        _string if data.is_string() => {
            if let Some(string) = data.as_str() {
                flattened.insert(parent.to_owned(), Value::String(string.to_owned()));
            }
        }
        _ => (),
    }
}
