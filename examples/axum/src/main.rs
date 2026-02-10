use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use complex::core::application::use_case::{
    todo::{CreateTodoUseCase, DeleteTodoUseCase, GetAllTodoUseCase, UpdateStatusTodoUseCase},
    user::{CreateUserUseCase, DeleteUserUseCase, GetAllUserUseCase, GetByIdUserUseCase},
};
use complex::core::domain::todo::Todo;
use complex::core::domain::user::User;
use sadi::Injector;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    injector: Arc<Injector>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateTodoRequest {
    user_id: i64,
    title: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateTodoStatusRequest {
    completed: bool,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    fn error(msg: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

// User Handlers
async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<User>>), (StatusCode, String)> {
    let create_user = state
        .injector
        .try_resolve::<CreateUserUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let user = create_user
        .execute(req.name, req.email)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::ok(user))))
}

async fn get_all_users(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<User>>>, (StatusCode, String)> {
    let get_all = state
        .injector
        .try_resolve::<GetAllUserUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let users = get_all
        .execute()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse::ok(users)))
}

async fn get_user_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, String)> {
    let get_by_id = state
        .injector
        .try_resolve::<GetByIdUserUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let user = get_by_id
        .execute(id as u32)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "User not found".to_string()))?;

    Ok(Json(ApiResponse::ok(user)))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<ApiResponse<bool>>), (StatusCode, String)> {
    let delete = state
        .injector
        .try_resolve::<DeleteUserUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let deleted = delete
        .execute(id as u32)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok(deleted))))
}

// Todo Handlers
async fn create_todo(
    State(state): State<AppState>,
    Json(req): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<ApiResponse<Todo>>), (StatusCode, String)> {
    let create_todo = state
        .injector
        .try_resolve::<CreateTodoUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let todo = create_todo
        .execute(req.user_id as u32, req.title, req.description)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::ok(todo))))
}

async fn get_all_todos(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Todo>>>, (StatusCode, String)> {
    let get_all = state
        .injector
        .try_resolve::<GetAllTodoUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let todos = get_all
        .execute()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(ApiResponse::ok(todos)))
}

async fn update_todo_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTodoStatusRequest>,
) -> Result<Json<ApiResponse<Todo>>, (StatusCode, String)> {
    let update = state
        .injector
        .try_resolve::<UpdateStatusTodoUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let todo = update
        .execute(id as u32, req.completed)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Todo not found".to_string()))?;

    Ok(Json(ApiResponse::ok(todo)))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<ApiResponse<bool>>), (StatusCode, String)> {
    let delete = state
        .injector
        .try_resolve::<DeleteTodoUseCase>()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to resolve use case: {:?}", e),
            )
        })?;

    let deleted = delete
        .execute(id as u32)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok(deleted))))
}

async fn health_check() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build the application with dependency injection
    let app_di = complex::infra::di::build().expect("Failed to build application");
    let state = AppState {
        injector: app_di.injector().clone(),
    };

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // User routes
        .route("/users", post(create_user))
        .route("/users", get(get_all_users))
        .route("/users/{id}", get(get_user_by_id))
        .route("/users/{id}", delete(delete_user))
        // Todo routes
        .route("/todos", post(create_todo))
        .route("/todos", get(get_all_todos))
        .route("/todos/{id}/status", put(update_todo_status))
        .route("/todos/{id}", delete(delete_todo))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    println!("🚀 Server running on http://127.0.0.1:3000");
    println!("📚 Available endpoints:");
    println!("  GET    /health");
    println!("  POST   /users");
    println!("  GET    /users");
    println!("  GET    /users/:id");
    println!("  DELETE /users/:id");
    println!("  POST   /todos");
    println!("  GET    /todos");
    println!("  PUT    /todos/:id/status");
    println!("  DELETE /todos/:id");

    axum::serve(listener, app).await.expect("Server error");
}
