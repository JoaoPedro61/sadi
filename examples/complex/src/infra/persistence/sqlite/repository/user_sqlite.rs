use std::sync::Arc;

use crate::core::domain::user::{User, UserRepository};
use crate::infra::persistence::sqlite::SqliteClient;

pub struct UserSqliteRepository {
    sqlite_client: Arc<SqliteClient>,
}

impl UserSqliteRepository {
    pub fn new(sqlite_client: Arc<SqliteClient>) -> Self {
        Self { sqlite_client }
    }
}

#[async_trait::async_trait]
impl UserRepository for UserSqliteRepository {
    async fn get_all(&self) -> Result<Vec<User>, String> {
        todo!()
    }

    async fn get_by_id(&self, id: u32) -> Result<Option<User>, String> {
        todo!()
    }

    async fn create(&self, name: String, email: String) -> Result<User, String> {
        todo!()
    }

    async fn delete(&self, id: u32) -> Result<bool, String> {
        todo!()
    }
}
