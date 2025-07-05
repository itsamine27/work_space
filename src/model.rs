use crate::error::Result as Myresult;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as, query_scalar, PgPool};
use crate::error::Error;
use std::result::Result;
use sqlx::Error as SqlxError;
//-------------------------
// Database-backed Models
//-------------------------

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub pwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub descp: String,
    pub work_id: i64,
}

//-------------------------
// Input (Form) Structs
//-------------------------

#[derive(Deserialize)]
pub struct CreateWork {
    pub name: String,
}

#[derive(Deserialize)]
pub struct CrUser {
    pub name: String,
    pub pwd: String,
    pub work_id: i64,
}

#[derive(Deserialize)]
pub struct CreateTask {
    pub name: String,
    pub descp: String,
    pub work_id: i64,
}

//-------------------------
// Controller
//-------------------------

#[derive(Clone)]
pub struct ModelController {
    pub pool: PgPool,
}

impl ModelController {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn newwork(&self, name: &str) -> Result<Workspace, SqlxError> {
        let ws = query_as!(
            Workspace,
            r#"
            INSERT INTO work_space (name)
            VALUES ($1)
            RETURNING id, name
            "#,
            name
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn allwork(&self) -> Result<Vec<Workspace>, SqlxError> {
        let ws = query_as!(
            Workspace,
            r#"
            SELECT id, name
            FROM work_space
            ORDER BY id
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn find_work(&self, id: i32) -> Myresult<bool> {
        // query_scalar! is for a single-column primitive (here: bool)
        let exists: bool = query_scalar!(
            r#"
            SELECT EXISTS(
            SELECT 1
            FROM work_space
            WHERE id = $1
            )
            "#,
            id             // ← $1 binds to this i64
        )
        .fetch_one(&self.pool)  
        .await?
        .unwrap();                // maps sqlx::Error → SqlxError

        Ok(exists)
    }


    pub async fn newtask(&self, t: &CreateTask) -> Myresult<Task> {
        // ensure workspace exists
        if !self.find_work(t.work_id as i32).await? {
            return Err(Error::Workspacenotfound);
        }
        let task = query_as!(
            Task,
            r#"
            INSERT INTO tasks (name, description, work_id)
            VALUES ($1, $2, $3)
            RETURNING id, name, description AS "descp!", work_id
            "#,
            t.name,
            t.descp,
            t.work_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(task)
    }

    pub async fn createuser(&self, u: &CrUser) -> Myresult<User> {
        // ensure workspace exists
        if !self.find_work(u.work_id as i32).await? {
            return Err(Error::Workspacenotfound);
        }

        // insert into users table
        let user = query_as!(
            User,
            r#"
            INSERT INTO "user" (name, password)
            VALUES ($1, $2)
            RETURNING id, name, password AS "pwd!"
            "#,
            u.name,
            u.pwd
        )
        .fetch_one(&self.pool)
        .await?;

        // link user to workspace
        query!(
            r#"
            INSERT INTO work_user (user_id, work_id)
            VALUES ($1, $2)
            "#,
            user.id as i32,
            u.work_id as i32
        )
        .execute(&self.pool)
        .await?;

        Ok(user)
    }
}
