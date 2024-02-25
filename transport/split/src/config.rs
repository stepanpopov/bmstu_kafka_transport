use std::{env::var_os, ffi::OsStr};

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            listen: env_or("LISTEN", "0.0.0.0:8000".to_owned()),
        }
    }
}

fn env_or<K: AsRef<OsStr>>(
    key: K,
    default: String,
) -> String {
    var_os(key)
        .map(|os_str| os_str.into_string().unwrap())
        .unwrap_or(default)
}
