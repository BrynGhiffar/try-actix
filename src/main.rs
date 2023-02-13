use std::{time::{UNIX_EPOCH, SystemTime}, sync::Mutex};

use actix_web::{ get, web, App, HttpServer, Responder, middleware, HttpResponse };

use service::{find_user_by_username, find_user_by_email};
use sha2::Sha256;
use hmac::{Hmac, Mac , digest::InvalidLength };
use serde::{Serialize, Deserialize};
use jwt::SignWithKey;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    user_id: Option<String>,
    username: String,
    email: String,
    description: String,
    password: String
    // user_hobbies: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserLoginForm {
    email: String,
    password: String
}


#[derive(Serialize, Deserialize, Clone)]
pub struct UserLoginResponse {
    token: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserRegisterResponse {
    user_id: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserRegistrationForm {
    username: String,
    email: String,
    password: String,
    password_again: String,
}

struct ApplicationState {
    users: Mutex<Vec<User>>,
}

mod service {
    use std::sync::MutexGuard;

    use rand::{thread_rng, Rng};

    use crate::User;
    pub fn find_all_user(app_data: &MutexGuard<Vec<User>>) -> Vec<User> {
        return (*app_data).clone();
    }

    pub fn find_user_by_email(user_data: &MutexGuard<Vec<User>>, email: &String) -> Option<User> {
        let users = find_all_user(user_data);
        let mut user: Vec<User> = users.into_iter()
            .filter(|u| u.email == *email)
            .collect();
        return user.pop();
    }

    pub fn find_user_by_username(user_data: &MutexGuard<Vec<User>>, username: &String) -> Option<User> {

        let users = find_all_user(user_data);
        let mut user: Vec<User> = users.into_iter()
            .filter(|u| u.username == *username)
            .collect();
        return user.pop();
    }

    pub fn find_user_by_id(app_data: &MutexGuard<Vec<User>>, user_id: String) -> Option<User> {
        let users = find_all_user(app_data);
        let mut user: Vec<User> = users.into_iter()
                .filter_map(|u| match u.user_id {
                    Some(ref uid) if *uid == user_id => Some(u),
                    _ => None
                })
                .collect();
        let user = user.pop();
        return user;
    }

    pub fn create_user(app_data: &mut MutexGuard<Vec<User>>, user: User) -> Option<User> {
        let mut rng = thread_rng();
        let user_id = String::from("user:") + &rng.gen_range(1000..9999).to_string();
        let user = User {
            user_id: Some(user_id),
            ..user
        };
        app_data.push(user.clone());
        return Some(user);
    }

    pub fn delete_user(app_data: &mut MutexGuard<Vec<User>>, user_id: String) -> Option<User> {
        let pos = app_data.iter().position(|u| match u.user_id {
            Some(ref uid) if *uid == user_id => true,
            _ => false
        });
        let pos = pos?;
        let user = app_data.remove(pos);
        return Some(user);
    }

    pub fn update_user(user_data: &mut MutexGuard<Vec<User>>, user_id: String, user: User) -> Option<User> {
        let pos = user_data.iter().position(|u| match u.user_id {
            Some(ref uid) if *uid == user_id => true,
            _ => false
        });
        let pos = pos?;
        let old_user: &mut User = user_data.get_mut(pos)?;
        *old_user = User {
            user_id: Some(user_id),
            ..user
        };
        return Some((*old_user).clone());
    }
}

async fn find_all_user(data: web::Data<ApplicationState>) -> impl Responder {
    let app_data = data.users.lock().unwrap();
    let users = service::find_all_user(&app_data);
    HttpResponse::Ok()
        .json(users)
}

async fn find_user_by_id(user_id: web::Path<String>, data: web::Data<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let app_data = data.users.lock().unwrap();
    let user = service::find_user_by_id(&app_data, user_id);
    if let Some(user) = user {
        return HttpResponse::Ok()
            .json(user);
    } else {
        return HttpResponse::NotFound().body("user not found");
    }
}

async fn create_user(user: web::Json<User>, data: web::Data<ApplicationState>) -> impl Responder {
    let user = user.into_inner();
    let mut app_data = data.users.lock().unwrap();
    let user = service::create_user(&mut app_data, user);

    if let Some(user) = user {
        return HttpResponse::Ok()
            .json(user);
    } else {
        return HttpResponse::InternalServerError().body("An internal server error occurred");
    }
}

async fn delete_user(user_id: web::Path<String>, data: web::Data::<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let mut users = data.users.lock().unwrap();
    let user = service::delete_user(&mut users, user_id);

    if let Some(user) = user {
        return HttpResponse::Ok().json(user);
    } else {
        return HttpResponse::NotFound().body("user was not found");
    }
}

async fn update_user(user_id: web::Path<String>, body: web::Json<User>, data: web::Data::<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let new_user = body.into_inner();
    let mut users = data.users.lock().unwrap();
    let user = service::update_user(&mut users, user_id, new_user);
    if let Some(user) = user {
        return HttpResponse::Ok().json(user);
    } else {
        return HttpResponse::NotFound().body("user was not found");
    }
}

async fn login(body: web::Form<UserLoginForm>, data: web::Data::<ApplicationState>) -> impl Responder {
    let incorrect_email_response = HttpResponse::Ok().body("Incorrect Email Response");
    let incorrect_password_response = HttpResponse::Ok().body("Incorrect password response");
    let internal_server_error_response = HttpResponse::InternalServerError().body("An internal server error occurred");



    let user_form = body.into_inner();
    let UserLoginForm {
        email: form_email, password: form_password
    } = user_form;
    let users = data.users.lock().unwrap();
    let user = service::find_user_by_email(&users, &form_email);
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

async fn register(body: web::Form<UserRegistrationForm>, data: web::Data<ApplicationState>) -> impl Responder {
    let username_already_exists_response = HttpResponse::BadRequest().body("username already exists");
    let email_already_exists_response = HttpResponse::BadRequest().body("email already exists");
    let different_password_response = HttpResponse::BadRequest().body("password is different");
    let failed_to_create_user_response = HttpResponse::BadRequest().body("failed to create user");

    let user_form = body.into_inner();
    let UserRegistrationForm {
        username, email, password, password_again
    } = user_form;
    let mut user_data = data.users.lock().unwrap();

    // check if username is available
    let username_check_user = find_user_by_username(&user_data, &username);
    if let Some(_) = username_check_user { return username_already_exists_response };
    // check if email is available
    let email_check_user = find_user_by_email(&user_data, &email);
    if let Some(_) = email_check_user  { return email_already_exists_response; };
    // check if password and password again is the same
    if password != password_again { return different_password_response; };

    // create new user
    let password = sha256::digest(password);
    let user = User { user_id: None, username, email, description: "".to_string(), password };
    let Some(user) = service::create_user(&mut user_data, user) else {
        return failed_to_create_user_response;
    };

    let Some(user_id) = user.user_id else { return failed_to_create_user_response };

    return HttpResponse::Ok().json(UserRegisterResponse { user_id });
}

async fn healthcheck() -> impl Responder {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
                                .map(|t| t.as_secs())
                                .unwrap();
    format!("Server is ok: {timestamp}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let user_data = web::Data::new(ApplicationState {
        users: Mutex::new(vec![
        ])
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(user_data.clone())
            .service(greet)
            .route("/form/user/register", web::post().to(register))
            .route("/form/user/login", web::post().to(login))
            .route("/", web::get().to(healthcheck))
            .route("/user", web::get().to(find_all_user))
            .route("/user/{user_id}", web::get().to(find_user_by_id))
            .route("/user", web::post().to(create_user))
            .route("/user/{user_id}", web::delete().to(delete_user))
            .route("/user/{user_id}", web::put().to(update_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
