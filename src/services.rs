pub mod auth;
pub mod items;

#[derive(Clone)]
pub struct UserKey {
    pub auth_key: String,
}

impl UserKey {
    pub fn new() -> Self {
        UserKey { auth_key }
    }
}
