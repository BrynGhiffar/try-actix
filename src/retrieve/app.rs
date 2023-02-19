pub mod user {
    use crate::types::*;
    use std::sync::MutexGuard;

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
}
