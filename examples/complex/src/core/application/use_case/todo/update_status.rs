use std::sync::Arc;

use crate::core::domain::todo::{Todo, TodoRepository};

pub struct UpdateStatusTodoUseCase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl UpdateStatusTodoUseCase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, todo_id: u32, completed: bool) -> Result<Option<Todo>, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to update the todo status.
        self.todo_repository.update_status(todo_id, completed).await
    }
}
