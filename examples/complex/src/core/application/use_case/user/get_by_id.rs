use std::sync::Arc;

use crate::core::domain::user::{User, UserRepository};

pub struct GetByIdUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl GetByIdUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: u32) -> Result<Option<User>, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to get the user by ID.
        self.user_repository.get_by_id(user_id).await
    }
}
