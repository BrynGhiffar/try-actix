use crate::types::*;

use sha2::Sha256;
use hmac::{Hmac, Mac , digest::InvalidLength };
use jwt::SignWithKey;
use actix_web::{ web, Responder, HttpResponse };
use crate::retrieve::app as retr;


pub async fn login(body: web::Form<UserLoginForm>, data: web::Data::<ApplicationState>) -> impl Responder {

    let incorrect_email_response = HttpResponse::Ok().body("Incorrect Email Response");
    let incorrect_password_response = HttpResponse::Ok().body("Incorrect password response");
    let internal_server_error_response = HttpResponse::InternalServerError().body("An internal server error occurred");



    let user_form = body.into_inner();
    let UserLoginForm {
        email: form_email, password: form_password
    } = user_form;
    let users = data.users.lock().unwrap();
    let user = retr::user::find_user_by_email(&users, &form_email);
    let Some(user) = user else { return incorrect_email_response; };
    let form_password_hash = sha256::digest(form_password);
    let password_hash = user.password;
    if password_hash != form_password_hash {
        return incorrect_password_response;
    }


    // generate jwt token
    type HmacSha256 = Hmac<Sha256>;
    let key = std::env::var("TOKEN_SECRET").ok()
        .unwrap_or("abc123".to_string());
    let key = key
        .as_bytes();
    let Ok(secret_key): Result<HmacSha256, InvalidLength> = Hmac::new_from_slice(key) else {
        return internal_server_error_response;
    };
    let user_id = user.user_id;
    let Ok(token_str) = user_id.sign_with_key(&secret_key) else {
        return internal_server_error_response;
    };

    let response_body = UserLoginResponse { token: token_str };

    return HttpResponse::Ok().json(response_body);
}