use serde::{ Serialize, Deserialize };
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone)]
pub struct List<T> {
    pub user_id: String,
    pub list_id: Option<String>,
    pub list: T
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: Option<String>,
    pub username: String,
    pub email: String,
    pub description: String,
    pub password: String
    // user_hobbies: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserLoginForm {
    pub email: String,
    pub password: String
}


#[derive(Serialize, Deserialize, Clone)]
pub struct UserLoginResponse {
    pub token: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserRegisterResponse {
    pub user_id: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserRegistrationForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_again: String,
}

pub struct ApplicationState {
    pub users: Mutex<Vec<User>>,
}