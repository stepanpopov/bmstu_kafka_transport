use anyhow::{anyhow, Error};
use reqwest::{Client, IntoUrl, Url};

use super::consumer::Message;

pub struct MessageSender {
    client: Client,
    url: Url,
}

impl MessageSender {
    pub fn new(receive_url: impl IntoUrl) -> Result<Self, Error> {
        let url = receive_url.into_url()?;

        Ok(Self {
            client: reqwest::Client::new(),
            url,
        })
    }

    pub async fn send_message(&self, message: &Message) -> Result<(), Error> {
        match self
            .client
            .post(self.url.clone())
            .json(&message)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(e)),
        }
    }
}
