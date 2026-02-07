use std::sync::Arc;

use crate::core::domain::todo::{Todo, TodoRepository};

pub struct GetByIdTodoUseCase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl GetByIdTodoUseCase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, todo_id: u32) -> Result<Option<Todo>, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to get the todo by id.
        self.todo_repository.get_by_id(todo_id).await
    }
}
