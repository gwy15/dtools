use crate::config::Notification as Config;
use anyhow::Result;
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

pub struct Notifier {
    config: Config,
}
impl Notifier {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    #[allow(unused)]
    pub async fn notify(&self, receiver: String, title: String, body: String) -> Result<()> {
        // DEBUG
        return Ok(());
        let message = Message::builder()
            .from(Mailbox::new(None, self.config.sender.parse()?))
            .to(Mailbox::new(None, receiver.parse()?))
            .subject(title)
            .body(body)?;
        let credentials = Credentials::new(self.config.sender.clone(), self.config.pswd.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)?
            .credentials(credentials)
            .build();

        let r = mailer.send(message).await?;

        info!("send email to {}: {:?}", receiver, r);

        Ok(())
    }
}

#[tokio::test]
async fn test_send_mail() {
    let config = crate::config::Config::new().unwrap();
    let notifier = Notifier::new(config.notification);
    notifier
        .notify(
            config.master,
            "成功跑通啦".to_string(),
            "这是内文".to_string(),
        )
        .await
        .unwrap();
}
