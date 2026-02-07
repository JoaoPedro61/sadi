use std::sync::Arc;

use crate::core::domain::todo::TodoRepository;

pub struct DeleteTodoUseCase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl DeleteTodoUseCase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, id: u32) -> Result<bool, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to delete the todo.
        self.todo_repository.delete(id).await
    }
}
