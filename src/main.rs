use std::{env, net::SocketAddr};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use axum::{
    async_trait, extract::{FromRequestParts, Path}, http::{request::Parts, StatusCode}, response::{Html, IntoResponse, Response}, routing::{get, post}, serve, Router
};
use headers::{Cookie, HeaderMapExt};
mod api_ws;
mod error;
mod model;

use api_ws::ws_fun::{create_workspace_handler, list_workspaces_handler, signup_handler, create_task_handler};
use model::ModelController;
//cookie imports 
use axum::http::{header, HeaderMap, HeaderValue};
use cookie::{Cookie as Requescookie, SameSite};
use dotenv::dotenv;
use crate::error::Error;



#[tokio::main]
async fn main() {
    // 1) Initialize your controllers
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DB url not set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("");
    // 2) Build your router
    let mc = ModelController::new(pool.clone());
    let app = Router::new()
        .route("/hello/:name", get(hello))
        .route("/api/newwork", post(create_workspace_handler))
        .route("/api/signup",  post(signup_handler))
        .route("/api/allwork", get(list_workspaces_handler))
        .route("/api/newtask", post(create_task_handler))
        .with_state(mc);

    // 3) Bind and serve using axum::serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}

async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    Html(format!("Hello, {}!", name))
}
async fn set_cookie(value: &str) -> (HeaderMap, &'static str) {
    
    let mut cookie = Requescookie::new("workspace_id", value);
    cookie.set_path("/");
    cookie.set_same_site(SameSite::Lax);
    cookie.set_http_only(true);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );

    (headers, "workspace_id cookie set")
}
pub struct Onlyworker{
    pub work_id: u64,
}

#[async_trait]
impl FromRequestParts<ModelController> for Onlyworker
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, state: &ModelController) -> Result<Self, Self::Rejection>{
        let cookies = parts.headers.typed_get::<Cookie>().ok_or_else(|| {
            (StatusCode::BAD_REQUEST, "Missing Cookie header").into_response()
        })?;
        let Some(work_id) = cookies.get("workspace_id") else {
            return Err(Error::Workspacenotfound.into_response());
        };
        let res = work_id.parse::<u64>().map_err(|_| {
            (StatusCode::BAD_REQUEST, "Invalid workspace_id cookie").into_response()
        })?;
        state.find_work(res as i32).await.map_err(|e| e.into_response())?;
        Ok(Onlyworker {work_id: res})
    }
}