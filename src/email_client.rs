use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender
        }
    }
    pub async fn send_mail(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_context: &str,
        text_content: &str
    ) -> Result<(), String> {
        todo!()
    }
}