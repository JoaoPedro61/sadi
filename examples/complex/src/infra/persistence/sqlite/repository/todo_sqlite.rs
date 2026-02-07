use std::sync::Arc;

use crate::core::domain::todo::{Todo, TodoRepository};
use crate::infra::persistence::sqlite::SqliteClient;

pub struct TodoSqliteRepository {
    sqlite_client: Arc<SqliteClient>,
}

impl TodoSqliteRepository {
    pub fn new(sqlite_client: Arc<SqliteClient>) -> Self {
        Self { sqlite_client }
    }
}

#[async_trait::async_trait]
impl TodoRepository for TodoSqliteRepository {
    async fn get_all(&self) -> Result<Vec<Todo>, String> {
        todo!()
    }

    async fn get_by_id(&self, id: u32) -> Result<Option<Todo>, String> {
        todo!()
    }

    async fn create(
        &self,
        user_id: u32,
        title: String,
        description: String,
    ) -> Result<Todo, String> {
        todo!()
    }

    async fn update_status(&self, id: u32, completed: bool) -> Result<Option<Todo>, String> {
        todo!()
    }

    async fn delete(&self, id: u32) -> Result<bool, String> {
        todo!()
    }
}
