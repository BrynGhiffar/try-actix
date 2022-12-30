use std::{time::{UNIX_EPOCH, SystemTime}, sync::Mutex};

use actix_web::{ get, web, App, HttpServer, Responder, middleware, HttpResponse };

use serde::{Serialize, Deserialize};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    user_id: Option<String>,
    user_name: String,
    user_description: String,
    // user_hobbies: Vec<String>,
}

struct ApplicationState {
    users: Mutex<Vec<User>>,
}

mod service {
    use std::sync::MutexGuard;

    use rand::{thread_rng, Rng};

    use crate::User;
    pub fn find_all_user(app_data: MutexGuard<Vec<User>>) -> Vec<User> {
        return (*app_data).clone();
    }

    pub fn find_user_by_id(app_data: MutexGuard<Vec<User>>, user_id: String) -> Option<User> {
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
    let users = service::find_all_user(app_data);
    HttpResponse::Ok()
        .json(users)
}

async fn find_user_by_id(user_id: web::Path<String>, data: web::Data<ApplicationState>) -> impl Responder {
    let user_id = user_id.to_string();
    let app_data = data.users.lock().unwrap();
    let user = service::find_user_by_id(app_data, user_id);
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
            User {
                user_id: Some(String::from("user:1")),
                user_name: String::from("Joe"),
                user_description: String::from("Your average joe")
            },
            User {
                user_id: Some(String::from("user:2")),
                user_name: String::from("Max"),
                user_description: String::from("Maximummm!!")
            },
            User {
                user_id: Some(String::from("user:3")),
                user_name: String::from("Bryan"),
                user_description: String::from("The Amazing Bryan")
            }
        ])
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(user_data.clone())
            .service(greet)
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
