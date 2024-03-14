use std::borrow::Borrow;

use clap::{Arg, Command};

pub struct Config {
    pub brokers: String,
    pub group_id: String,
    pub topic: String,
    pub receive_url: String,
}

impl Config {
    pub fn from_cmd() -> Self {
        let matches = Config::command().get_matches();
        let topic = matches.get_one::<String>("topic").unwrap().to_owned();
        let brokers = matches.get_one::<String>("brokers").unwrap().to_owned();
        let group_id = matches.get_one::<String>("group-id").unwrap().to_owned();
        let receive_url = matches.get_one::<String>("receive_url").unwrap().to_owned();

        Self {
            topic,
            brokers,
            group_id,
            receive_url,
        }
    }

    fn command() -> Command {
        Command::new("consumer")
            .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
            .about("Simple command line consumer")
            .arg(
                Arg::new("brokers")
                    .short('b')
                    .long("brokers")
                    .help("Broker list in kafka format")
                    .default_value("localhost:9092"),
            )
            .arg(
                Arg::new("group-id")
                    .short('g')
                    .long("group-id")
                    .help("Consumer group id")
                    .default_value("example_consumer_group_id"),
            )
            .arg(
                Arg::new("topic")
                    .short('t')
                    .long("topic")
                    .help("Topic list")
                    .required(true),
            )
            .arg(
                Arg::new("receive_url")
                    .short('r')
                    .long("receive_url")
                    .help("url of receive service")
                    .required(true),
            )
    }
}
