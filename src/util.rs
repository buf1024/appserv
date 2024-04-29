use data_encoding::HEXLOWER;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use ring::digest::{Context, SHA256};

use crate::{config::CONFIG, errors::Error, Result};

pub fn send_email(receiver: String, subject: String, body: String) -> Result {
    if CONFIG.smtp_host.is_none() || CONFIG.smtp_passwd.is_none() || CONFIG.smtp_sender.is_none() {
        tracing::error!("smtp configuration error");
        return Err(Error::Internal("smtp configuration error".to_string()));
    }
    let sender_email = CONFIG.smtp_sender.clone().unwrap();
    let name = sender_email.split("@").nth(0).unwrap().to_string();
    let sender = format!("{name} <{sender_email}>");

    let email = Message::builder()
        .from(
            sender
                .parse()
                .map_err(|e| Error::Internal(format!("email sender format error: {e}")))?,
        )
        .to(receiver
            .parse()
            .map_err(|e| Error::Internal(format!("email sender format error: {e}")))?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body)
        .map_err(|e| Error::Internal(format!("build email body error: {e}")))?;

    let credentials = Credentials::new(sender_email, CONFIG.smtp_passwd.clone().unwrap());

    let mailer = SmtpTransport::relay(CONFIG.smtp_host.clone().unwrap().as_str())
        .map_err(|e| Error::Internal(format!("connect smtp server error: {e}")))?
        .credentials(credentials)
        .build();

    match mailer.send(&email) {
        Ok(_) => {
            tracing::info!("send email success: {}", receiver);
            Ok(())
        }
        Err(e) => {
            tracing::error!("send email error: {e}");
            Err(Error::SendEmail)
        }
    }
}

pub fn gen_passwd(email: &str, passwd: &str) -> String {
    let mut context = Context::new(&SHA256);
    let mut data = String::new();
    data.push_str(email);
    data.push_str(passwd);
    context.update(data.as_bytes());
    let digest = context.finish();
    HEXLOWER.encode(digest.as_ref())
}
