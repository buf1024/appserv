mod captcha;
pub use captcha::captcha;

mod send_email_code;
pub use send_email_code::send_email_code;


pub mod user;

pub mod hiqradio;

pub const COOKIE_NAME: &'static str = "SESSION";

pub fn ok_with_trace<T: core::fmt::Debug>(rsp: T) -> crate::Result<axum::Json<T>> {
    tracing::info!("\nrsp: {:?}\n", rsp);

    Ok(axum::Json(rsp))
}
