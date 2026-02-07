use std::sync::Arc;

use crate::core::domain::user::{User, UserRepository};

pub struct GetAllUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl GetAllUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self) -> Result<Vec<User>, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to delete the user.
        self.user_repository.get_all().await
    }
}
