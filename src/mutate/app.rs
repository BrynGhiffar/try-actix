pub mod user {
    use std::sync::MutexGuard;
    use rand::{thread_rng, Rng};
    use crate::types::*;

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
