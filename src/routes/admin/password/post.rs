use actix_web::{HttpResponse, web};
use secrecy::Secret;
use crate::session_state::TypedSession;
use crate::utils::{e500, see_other};
use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::routes::admin::dashboard::get_username;
use sqlx::PgPool;
use secrecy::ExposeSecret;
use actix_web_flash_messages::FlashMessage;

#[derive(serde::Deserialize)]
pub struct FormData {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>, 
}

pub async fn change_password(
    form: web::Data<FormData>,
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = session.get_user_id().map_err(e500)?;
    if user_id.is_none() {
        return Ok(see_other("/login"));
    };
    let user_id = user_id.unwrap();

    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    };

    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        return Ok(see_other("/admin/password"));
    }
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        FlashMessage::error(
            "You entered two different new passwords - the field values must match.",
        )
        .send();
    }
    let new_password = form.new_password.expose_secret();
    if !is_password_strong(&new_password) {
        FlashMessage::error("Password must be between 12 and 128 characters long.").send();
        return Ok(see_other("/admin/password"));
    }
    let username = get_username(user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.current_password.clone(),
    };
    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect.").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        }
    }
    todo!()
}

fn is_password_strong(password: &str) -> bool {
    password.len() >= 12 && password.len() <= 128
}