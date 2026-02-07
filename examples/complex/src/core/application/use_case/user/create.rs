use std::sync::Arc;

use crate::core::domain::user::{User, UserRepository};

pub struct CreateUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, name: String, email: String) -> Result<User, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to create the user.
        self.user_repository.create(name, email).await
    }
}
