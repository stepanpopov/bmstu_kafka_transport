use clap::{Arg, Command};

pub struct Config<'a> {
    pub brokers: &'a str,
    pub group_id: &'a str,
    pub topic: &'a str,
    pub receive_url: &'a str,
}

impl<'a> Config<'a> {
    pub fn from_cmd() -> Self {
        let matches = Config::command().get_matches();
        let topic: &str = *matches.get_one("topic").unwrap();
        let brokers: &str = *matches.get_one("brokers").unwrap();
        let group_id = *matches.get_one("group-id").unwrap();
        let receive_url = *matches.get_one("receive_url").unwrap();

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
