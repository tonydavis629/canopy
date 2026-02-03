use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

impl User {
    /// Creates a new user
    pub fn new(id: u64, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            roles: vec!["user".to_string()],
        }
    }

    /// Adds a role to the user
    pub fn add_role(&mut self, role: String) {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
    }

    /// Checks if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
}

/// User management service
pub struct UserService {
    users: HashMap<u64, User>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Creates a new user
    pub fn create_user(&mut self, username: String, email: String) -> Result<User, String> {
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        let id = self.users.len() as u64 + 1;
        let user = User::new(id, username, email);
        self.users.insert(id, user.clone());
        Ok(user)
    }

    /// Finds a user by ID
    pub fn find_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    /// Lists all users
    pub fn list_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    /// Updates user email
    pub fn update_email(&mut self, id: u64, email: String) -> Result<(), String> {
        match self.users.get_mut(&id) {
            Some(user) => {
                user.email = email;
                Ok(())
            }
            None => Err("User not found".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let mut service = UserService::new();
        let user = service.create_user("test_user".to_string(), "test@example.com".to_string()).unwrap();
        
        assert_eq!(user.username, "test_user");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_find_user() {
        let mut service = UserService::new();
        let user = service.create_user("test_user".to_string(), "test@example.com".to_string()).unwrap();
        let found = service.find_user(user.id);
        
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);
    }
}