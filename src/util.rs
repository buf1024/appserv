use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

use crate::{config::CONFIG, errors::Error, Result};

pub fn send_email(receiver: String, subject: String, body: String) -> Result {
    if CONFIG.smtp_host.is_none() || CONFIG.smtp_passwd.is_none() || CONFIG.smtp_sender.is_none() {
        tracing::error!("smtp configuration error");
        return Err(Error::Custom("smtp configuration error".to_string()));
    }
    let sender_email = CONFIG.smtp_sender.clone().unwrap();
    let name = sender_email.split("@").nth(0).unwrap().to_string();
    let sender = format!("{name} <{sender_email}>");

    let email = Message::builder()
        .from(
            sender
                .parse()
                .map_err(|e| Error::Custom(format!("email sender format error: {e}")))?,
        )
        .to(receiver
            .parse()
            .map_err(|e| Error::Custom(format!("email sender format error: {e}")))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| Error::Custom(format!("build email body error: {e}")))?;

    let credentials = Credentials::new(sender_email, CONFIG.smtp_passwd.clone().unwrap());

    let mailer = SmtpTransport::relay(CONFIG.smtp_host.clone().unwrap().as_str())
        .map_err(|e| Error::Custom(format!("connect smtp server error: {e}")))?
        .credentials(credentials)
        .build();

    match mailer.send(&email) {
        Ok(_) => {
            tracing::info!("send email success: {}", receiver);
            Ok(())
        }
        Err(e) => {
            tracing::error!("send email error: {e}");
            Err(Error::Custom("send email error".to_string()))
        }
    }
}
