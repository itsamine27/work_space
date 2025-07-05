use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::Error as SqlxError; // Alias to avoid name collision

/// Unified result type across the app using your custom error enum.
pub type Result<T> = std::result::Result<T, Error>;

/// Application-level errors that map to meaningful HTTP responses.
#[derive(Debug)]
pub enum Error {
    Workspacenotfound,
    Sqlx(SqlxError),
}
impl From<SqlxError> for Error {
    fn from(e: SqlxError) -> Self {
        Error::Sqlx(e)
    }
}

/// Maps each `Error` into an HTTP response with a status code and JSON body.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Error::Workspacenotfound => (StatusCode::NOT_FOUND, "Workspace not found".to_string()),
            Error::Sqlx(ref err) => {
                eprintln!("💥 SQLx error: {err}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
        };

        let body = Json(json!({ "error": message }));

        (status, body).into_response()
    }
}