use clap::Parser;
use std::{env::var_os, ffi::OsStr};

const CODE_SERVICE_URL_DEFAULT: &str = "http://localhost:8080/code";
const LISTEN_DEFAULT: &str = "0.0.0.0:8000";
const CHUNK_BYTE_SIZE_DEFAULT: usize = 2;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    code_service_url: String,

    #[arg(long)]
    chunk_byte_size: usize,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
    pub code_service_url: String,
    pub chunk_byte_size: usize,
}

impl Config {
    pub fn build() -> Self {
        Self {
            listen: LISTEN_DEFAULT.to_owned(),
            code_service_url: CODE_SERVICE_URL_DEFAULT.to_owned(),
            chunk_byte_size: CHUNK_BYTE_SIZE_DEFAULT.to_owned(),
        }
    }

    pub fn cmd_parse(mut self) -> Self {
        let args = Args::parse();
        self.code_service_url = args.code_service_url;
        self.chunk_byte_size = args.chunk_byte_size;
        self
    }

    pub fn get_env(mut self) -> Self {
        self.listen = env_or("LISTEN", LISTEN_DEFAULT.to_owned());
        self
    }
}

fn env_or<K: AsRef<OsStr>>(key: K, default: String) -> String {
    var_os(key)
        .map(|os_str| os_str.into_string().unwrap())
        .unwrap_or(default)
}
