use actix_web::{ web, HttpResponse, Responder };
use crate::types::{ ApplicationState, User, UserRegistrationForm, UserRegisterResponse };
use crate::retrieve::app as retr;
use crate::mutate::app as mutr;

pub async fn register(body: web::Form<UserRegistrationForm>, data: web::Data<ApplicationState>) -> impl Responder {
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
    let username_check_user = retr::user::find_user_by_username(&user_data, &username);
    if let Some(_) = username_check_user { return username_already_exists_response };
    // check if email is available
    let email_check_user = retr::user::find_user_by_email(&user_data, &email);
    if let Some(_) = email_check_user  { return email_already_exists_response; };
    // check if password and password again is the same
    if password != password_again { return different_password_response; };

    // create new user
    let password = sha256::digest(password);
    let user = User { user_id: None, username, email, description: "".to_string(), password };
    let Some(user) = mutr::user::create_user(&mut user_data, user) else {
        return failed_to_create_user_response;
    };

    let Some(user_id) = user.user_id else { return failed_to_create_user_response };

    return HttpResponse::Ok().json(UserRegisterResponse { user_id });
}

pub async fn find_all_user(data: web::Data<ApplicationState>) -> impl Responder {
    let app_data = data.users.lock().unwrap();
    let users = retr::user::find_all_user(&app_data);
    HttpResponse::Ok()
        .json(users)
}

pub async fn find_user_by_id(user_id: web::Path<String>, data: web::Data<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let app_data = data.users.lock().unwrap();
    let user = retr::user::find_user_by_id(&app_data, user_id);
    if let Some(user) = user {
        return HttpResponse::Ok()
            .json(user);
    } else {
        return HttpResponse::NotFound().body("user not found");
    }
}

pub async fn create_user(user: web::Json<User>, data: web::Data<ApplicationState>) -> impl Responder {
    let user = user.into_inner();
    let mut app_data = data.users.lock().unwrap();
    let user = mutr::user::create_user(&mut app_data, user);

    if let Some(user) = user {
        return HttpResponse::Ok()
            .json(user);
    } else {
        return HttpResponse::InternalServerError().body("An internal server error occurred");
    }
}

pub async fn delete_user(user_id: web::Path<String>, data: web::Data::<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let mut users = data.users.lock().unwrap();
    let user = mutr::user::delete_user(&mut users, user_id);

    if let Some(user) = user {
        return HttpResponse::Ok().json(user);
    } else {
        return HttpResponse::NotFound().body("user was not found");
    }
}

pub async fn update_user(user_id: web::Path<String>, body: web::Json<User>, data: web::Data::<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let new_user = body.into_inner();
    let mut users = data.users.lock().unwrap();
    let user = mutr::user::update_user(&mut users, user_id, new_user);
    if let Some(user) = user {
        return HttpResponse::Ok().json(user);
    } else {
        return HttpResponse::NotFound().body("user was not found");
    }
}