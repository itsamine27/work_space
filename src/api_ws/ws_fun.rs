
//-------------------------
// Route Handlers
//-------------------------


use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};

use crate::{error::Error, model::{CrUser, CreateTask, CreateWork, ModelController}, set_cookie, Onlyworker};

pub async fn create_workspace_handler(
    State(ctrl): State<ModelController>,
    Json(data): Json<CreateWork>,
) -> impl IntoResponse {
    if data.name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Workspace name cannot be empty").into_response();
    }

    match ctrl.newwork(&data.name).await {
        Ok(ws) => (StatusCode::CREATED, Json(ws)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create workspace").into_response(),
    }
}

pub async fn list_workspaces_handler(
    State(ctrl): State<ModelController>,
) -> impl IntoResponse {
    match ctrl.allwork().await {
        Ok(list) => (StatusCode::OK, Json(list)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch workspaces").into_response(),
    }
}


pub async fn create_task_handler(
    Onlyworker {work_id} : Onlyworker,
    State(ctrl): State<ModelController>,
    Json(data): Json<CreateTask>,
) -> impl IntoResponse {
    match ctrl.newtask(&data).await {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(Error::Workspacenotfound) => (StatusCode::BAD_REQUEST, "Workspace not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create task").into_response(),
    }
}
pub async fn signup_handler(
    State(ctrl): State<ModelController>,
    Json(data): Json<CrUser>,
) -> Response {
    match ctrl.createuser(&data).await {
        Ok(user) => {
            let (cookie_headers, _msg) = set_cookie(&data.work_id.to_string()).await;

            let mut response = (StatusCode::CREATED, Json(user)).into_response();
            for (k, v) in cookie_headers.iter() {
                response.headers_mut().insert(k, v.clone());
            }
            response
        }
        Err(e) => e.into_response(),
    }

}
