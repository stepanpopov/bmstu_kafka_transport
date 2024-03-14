use clap::Parser;
use std::{env::var_os, ffi::OsStr, fmt::Display};

const LISTEN_DEFAULT: &str = "0.0.0.0:8002";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    brokers: String,

    #[arg(short, long)]
    topic: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub listen: String,
    pub brokers: String,
    pub topic: String,
}

impl Config {
    pub fn build() -> Self {
        Self {
            listen: LISTEN_DEFAULT.to_owned(),
            brokers: "".to_string(),
            topic: "".to_string(),
        }
    }

    pub fn cmd_parse(mut self) -> Self {
        let args = Args::parse();
        self.brokers = args.brokers;
        self.topic = args.topic;
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
