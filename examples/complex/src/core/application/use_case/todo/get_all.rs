use std::sync::Arc;

use crate::core::domain::todo::{Todo, TodoRepository};

pub struct GetAllTodoUseCase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl GetAllTodoUseCase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self) -> Result<Vec<Todo>, String> {
        // Business logic can be added here (e.g., validation, logging, etc.)
        // For simplicity, we directly call the repository to get all todos.
        self.todo_repository.get_all().await
    }
}
