use std::sync::Arc;

use crate::core::domain::user::UserRepository;

pub struct DeleteUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

impl DeleteUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: u32) -> Result<bool, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to delete the user.
        self.user_repository.delete(user_id).await
    }
}
