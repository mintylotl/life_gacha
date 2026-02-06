use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use chrono::{DateTime, TimeZone, Timelike, Utc};
use gacha_protocol::{self, PityCtx, Rarities, roll};
use rusqlite::Error as rusqError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Error as serdeError;
use serde_json::{self};
use std::any::Any;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tower_http::cors::CorsLayer;

#[derive(thiserror::Error, Debug)]
enum PersistenceError {
    #[error("Serialization of User Error")]
    JsonError(#[from] serdeError),
    #[error("Sqlite user loading error!")]
    DBError(#[from] rusqError),
    #[error("User Not Found")]
    NotFound,
}
struct SqliteRepo {
    db: Mutex<Connection>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserId(String);
#[derive(Debug, Deserialize, Serialize)]
struct Voucher {}
#[derive(Debug, Deserialize, Serialize)]
enum Category {
    S_Node,
    A_Node,
    B_Node,
}

trait UserRepo {
    fn load(&self, id: UserId) -> Result<User, PersistenceError>;
    fn save(&self, user: &User) -> Result<(), PersistenceError>;
}
#[derive(Serialize, Deserialize)]
struct User {
    id: UserId,
    astrai: u64,
    astrum: u64,
    username: String,
    email: Option<String>,
    vouchers: Option<Vec<Voucher>>,
    active_timer: bool,
    timer: Option<Timer>,
    sss_pity: u16,
    s_pity: u16,
    a_pity: u16,
    total_pulls: u128,
}
impl UserRepo for SqliteRepo {
    fn save(&self, user: &User) -> Result<(), PersistenceError> {
        let user_bytes = serde_json::to_string_pretty(user)?;
        self.db.lock().unwrap().execute(
            "INSERT OR REPLACE INTO users (id, data) VALUES (?1, ?2)",
            params![user.id.0, user_bytes],
        )?;
        Ok(())
    }
    fn load(&self, id: UserId) -> Result<User, PersistenceError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT data FROM users WHERE id = ?1")?;

        let user_json: String =
            stmt.query_row(params![id.0], |row| row.get(0))
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => PersistenceError::NotFound,
                    _ => PersistenceError::DBError(e),
                })?;

        let user: User = serde_json::from_str(&user_json)?;
        Ok(user)
    }
}
impl SqliteRepo {
    fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("Error opening DB!");
        let user = User {
            id: UserId("axol999".to_string()),
            astrum: 1600,
            astrai: 2,
            email: None,
            active_timer: false,
            timer: None,
            a_pity: 0,
            s_pity: 0,
            sss_pity: 0,
            total_pulls: 0,
            username: "Axol".to_string(),
            vouchers: None,
        };
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
            [],
        );

        let _ = conn.execute(
            "INSERT OR REPLACE INTO users (id, data) Values(?1, ?2)",
            params![user.id.0, serde_json::to_string(&user).unwrap()],
        );
        Self {
            db: Mutex::new(conn),
        }
    }
}
#[derive(Serialize)]
enum SerializedRarity {
    MythicSSS,
    S,
    A,
    B,
    NoTickets,
}
impl TryFrom<Rarities> for SerializedRarity {
    type Error = PersistenceError;
    fn try_from(value: Rarities) -> Result<Self, Self::Error> {
        match value {
            Rarities::MythicSSS => Ok(SerializedRarity::MythicSSS),
            Rarities::S => Ok(SerializedRarity::S),
            Rarities::A => Ok(SerializedRarity::A),
            Rarities::B => Ok(SerializedRarity::B),
        }
    }
}

#[derive(Deserialize)]
struct TimerRequest {
    userid: String,
    category: Category,
}
#[derive(Serialize)]
struct TimerResponse {
    status: String,
    timer_id: String,
    reward: Option<u64>,
}
#[derive(Serialize, Deserialize)]
struct Timer {
    id: String,
    owner: UserId,
    started: DateTime<Utc>,
    ended: Option<i64>,
    category: Category,
}
impl TimerResponse {
    fn new(status: String) -> Self {
        Self {
            status,
            timer_id: "".to_string(),
            reward: None,
        }
    }
    fn already_exists(id: String) -> Self {
        Self {
            status: "Timer Already Exists / Active".to_string(),
            timer_id: id,
            reward: None,
        }
    }
}

#[derive(Deserialize)]
struct PullRequest {
    userid: String,
}
#[derive(Serialize)]
struct PullResponse {
    result: SerializedRarity,
}
impl TryFrom<&User> for PityCtx {
    type Error = PersistenceError;
    fn try_from(user: &User) -> Result<Self, Self::Error> {
        Ok(PityCtx {
            sss_pity: user.sss_pity,
            s_pity: user.s_pity,
            a_pity: user.a_pity,
        })
    }
}

#[derive(Clone)]
struct AppState {
    repo: Arc<SqliteRepo>,
}

fn apply_outcome(user: &mut User, outcome: &Rarities) {
    println!(
        "User Pity stats: {}\n{}\n{}",
        user.sss_pity, user.s_pity, user.a_pity
    );
    match outcome {
        Rarities::MythicSSS => {
            user.sss_pity = 0;
            user.s_pity += 1;
            user.a_pity += 1;
        }
        Rarities::S => {
            user.s_pity = 0;
            user.sss_pity += 1;
            user.a_pity += 1;
        }
        Rarities::A => {
            user.a_pity = 0;
            user.sss_pity += 1;
            user.s_pity += 1;
        }
        Rarities::B => {
            user.sss_pity += 1;
            user.s_pity += 1;
            user.a_pity += 1;
        }
    }
    user.total_pulls += 1;
    if user.astrai > 0 {
        user.astrai -= 1;
        return;
    }
    user.astrum -= 160;
}

async fn handle_pull(
    State(state): State<AppState>,
    Json(req): Json<PullRequest>,
) -> Result<Json<PullResponse>, StatusCode> {
    let mut user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if user.astrai < 1 && user.astrum < 160 {
        return Ok(Json(PullResponse {
            result: SerializedRarity::NoTickets,
        }));
    }

    let pityctx = PityCtx::try_from(&user).unwrap();
    let outcome = roll(&pityctx);

    apply_outcome(&mut user, &outcome);
    let _ = state.repo.save(&user);

    Ok(Json(PullResponse {
        result: SerializedRarity::try_from(outcome).unwrap(),
    }))
}
fn gen_timer_id() -> String {
    "1111".to_string()
}

async fn start_timer(
    State(state): State<AppState>,
    Json(req): Json<TimerRequest>,
) -> Result<Json<TimerResponse>, StatusCode> {
    println!("{}, {:?}", req.userid, req.category);
    let mut user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if user.active_timer {
        return Ok(Json(TimerResponse::already_exists(user.timer.unwrap().id)));
    }

    user.timer = Some(Timer {
        category: req.category,
        ended: None,
        started: Utc::now(),
        id: gen_timer_id(),
        owner: user.id.clone(),
    });

    user.active_timer = true;
    let _ = state.repo.save(&mut user);

    Ok(Json(TimerResponse {
        status: "Request Accepted".to_string(),
        timer_id: user.timer.unwrap().id,
        reward: Some(0),
    }))
}

async fn stop_timer(
    State(state): State<AppState>,
    Json(req): Json<TimerRequest>,
) -> Result<Json<TimerResponse>, StatusCode> {
    let mut user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !user.active_timer {
        return Ok(Json(TimerResponse::new("No Active Timers".to_string())));
    }

    let timer: Timer = user.timer.unwrap();
    let reward = Some(resolve_timer(timer));
    user.active_timer = false;
    user.timer = None;
    user.astrum += reward.unwrap();
    let _ = state.repo.save(&user);

    Ok(Json(TimerResponse {
        status: format!("Timer Stopped"),
        timer_id: "".to_string(),
        reward,
    }))
}
fn resolve_timer(timer: Timer) -> u64 {
    let duration = Utc::now() - timer.started;
    let hours = duration.num_hours();
    let mut reward: i64 = 0;

    if hours < 1 {
        return 0;
    }

    match timer.category {
        Category::S_Node => {
            reward = hours * 160;
        }
        Category::A_Node => {
            reward = hours * 120;
        }
        Category::B_Node => {
            reward = hours * 60;
        }
    }
    reward as u64
}

#[tokio::main]
async fn main() {
    let repo = SqliteRepo::new("userdata.sql");
    let state = AppState { repo: repo.into() };

    let app = Router::new()
        .route("/start_timer", post(start_timer))
        .route("/pull", post(handle_pull))
        .route("/stop_timer", post(stop_timer))
        .layer(CorsLayer::permissive())
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
