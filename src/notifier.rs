use crate::config::Notification as Config;
use anyhow::Result;
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

pub struct Notifier {
    config: Config,
    send: bool,
}
impl Notifier {
    pub fn new(config: Config) -> Self {
        Self { config, send: true }
    }

    pub fn noop() -> Self {
        Self {
            config: Default::default(),
            send: false,
        }
    }

    #[allow(unused)]
    pub async fn notify(&self, receiver: &str, title: String, body: String) -> Result<()> {
        if !self.send {
            debug!("Notifier set as no-op. return.");
            return Ok(());
        }
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

        debug!("send email to {}: {:?}", receiver, r);

        info!("发送通知邮件到 {} 完成", receiver);

        Ok(())
    }
}
