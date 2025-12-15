//! Async dependency injection example for SaDi
//!
//! This example demonstrates how to use SaDi's async support with the unified Container API.

use sadi::Container;
use std::sync::Arc;

#[derive(Clone)]
struct DatabaseConnection {
    connection_string: String,
}

impl DatabaseConnection {
    async fn connect(url: &str) -> Self {
        // Simulate async connection setup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Self {
            connection_string: url.to_string(),
        }
    }

    fn query(&self, sql: &str) -> String {
        format!("Executing '{}' on {}", sql, self.connection_string)
    }
}

#[derive(Clone)]
struct UserRepository {
    db: Arc<DatabaseConnection>,
}

impl UserRepository {
    fn create_user(&self, name: &str) -> String {
        let sql = format!("INSERT INTO users (name) VALUES ('{}')", name);
        self.db.query(&sql)
    }
}

#[tokio::main]
async fn main() {
    // Create a single Arc-wrapped container
    let container = Arc::new(Container::new());

    // Register DatabaseConnection as a singleton async factory
    container
        .bind_async_concrete_singleton::<DatabaseConnection, DatabaseConnection, _, _>(|_| async {
            DatabaseConnection::connect("postgresql://localhost:5432/myapp").await
        })
        .await
        .expect("Failed to bind DatabaseConnection");

    // Register UserRepository with DatabaseConnection dependency
    container
        .bind_async_concrete::<UserRepository, UserRepository, _, _>(|c| async move {
            let db = c
                .resolve_async::<DatabaseConnection>()
                .await
                .expect("Failed to resolve DatabaseConnection");
            UserRepository { db }
        })
        .await
        .expect("Failed to bind UserRepository");

    // Resolve and use services
    let user_repo = container
        .clone()
        .resolve_async::<UserRepository>()
        .await
        .expect("Failed to resolve UserRepository");

    println!("{}", user_repo.create_user("Alice"));
    println!("{}", user_repo.create_user("Bob"));

    // Verify singleton behavior - database connection should be reused
    let db1 = container
        .clone()
        .resolve_async::<DatabaseConnection>()
        .await
        .expect("Failed to resolve DatabaseConnection");
    let db2 = container
        .clone()
        .resolve_async::<DatabaseConnection>()
        .await
        .expect("Failed to resolve DatabaseConnection");

    println!(
        "\nDatabase connections are the same singleton: {}",
        Arc::ptr_eq(&db1, &db2)
    );
}
