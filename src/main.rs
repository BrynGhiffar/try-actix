mod types;
mod retrieve;
mod mutate;
mod handler;

use types::*;
use actix_web::{ HttpServer, App, web, middleware };
use std::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let user_data = web::Data::new(ApplicationState {
        users: Mutex::new(vec![
        ])
    });

    std::thread::spawn(|| {
        loop {
            
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(user_data.clone())
            .route("/", web::get().to(handler::root::healthcheck))
            .service(
                web::scope("/form")
                    .route("/user/register", web::post().to(handler::user::register))
                    .route("/user/login", web::post().to(handler::auth::login))
            )
            .service(
                web::scope("/user")
                    .route("", web::get().to(handler::user::find_all_user))
                    .route("/{user_id}", web::get().to(handler::user::find_user_by_id))
                    .route("", web::post().to(handler::user::create_user))
                    .route("/{user_id}", web::delete().to(handler::user::delete_user))
                    .route("/{user_id}", web::put().to(handler::user::update_user))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
