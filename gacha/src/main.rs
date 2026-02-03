use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use rusqlite::Error as rusqError;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Error as serdeError;
use serde_json::{self, from_slice};
use std::iter::Fuse;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use gacha_protocol::{self, roll, PityCtx, Rarities};

#[derive(thiserror::Error, Debug)]
enum PersistenceError {
    #[error("Serialization of User Error")]
    JsonError(#[from] serdeError),
    #[error("Sqlite user loading error!")]
    DBError(#[from] rusqError),
    #[error("Couldn't make Rarity from Rarities")]
    SerializeRarityErr,
    #[error("User Not Found")]
    NotFound,
    #[error("Error creating PityCtx from User")]
    PityCtxErr
}
struct SqliteRepo {
    db: Mutex<Connection>,
}
#[derive(Debug, Deserialize, Serialize)]
struct UserId(String);
#[derive(Debug, Deserialize, Serialize)]
struct Voucher {}
//#[derive(Debug, Deserialize, Serialize)]
//struct TotalPullObj(u128, u128, u128, u128);

trait UserRepo {
    fn load(&self, id: UserId) -> Result<User, PersistenceError>;
    fn save(&self, user: &User) -> Result<(), PersistenceError>;
}
#[derive(serde::Serialize, serde::Deserialize)]
struct User {
    id: UserId,
    astrai: u64,
    astrum: u64,
    username: String,
    email: Option<String>,
    vouchers: Vec<Voucher>,
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

        // 1. Load as a String instead of Vec<u8>
        let user_json: String = stmt.query_row(params![id.0], |row| {
            row.get(0)
        }).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => PersistenceError::NotFound,
            _ => PersistenceError::DBError(e),
        })?;

        // 2. Deserialize directly from the String
        let user: User = serde_json::from_str(&user_json)?;
        Ok(user)
    }
}
impl SqliteRepo {
    fn new(path: &str) -> Self {
        let conn = Connection::open(path).unwrap();
        Self { db: Mutex::new(conn) }
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
            _ => Err(PersistenceError::SerializeRarityErr),
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
pub struct AppState {
    pub repo: Arc<SqliteRepo>,
}

fn apply_outcome(user: &mut User, outcome: &Rarities) {
    println!("User Pity stats: {}\n{}\n{}", user.sss_pity, user.s_pity, user.a_pity);
    match outcome {
        Rarities::MythicSSS => { user.sss_pity = 0; user.s_pity += 1; user.a_pity += 1;  }
        Rarities::S => { user.s_pity = 0; user.sss_pity += 1; user.a_pity += 1; }
        Rarities::A => { user.a_pity = 0; user.sss_pity += 1; user.s_pity += 1; }
        Rarities::B => { user.sss_pity += 1; user.s_pity += 1; user.a_pity += 1; }
    }
    user.total_pulls += 1;
    if user.astrai > 0 {
        user.astrai -= 1;
        return;
    }
    user.astrum -= 160;
}

async fn handle_pull(State(state): State<AppState>, Json(req): Json<PullRequest>) -> Json<PullResponse> {
    let mut user = state.repo.load(UserId(req.userid)).expect("Error loading user");
    if user.astrai < 1 && user.astrum < 160 {
        return Json(PullResponse { result: SerializedRarity::NoTickets })
    }

    let pityctx = PityCtx::try_from(&user).unwrap();
    let outcome = roll(&pityctx);

    apply_outcome(&mut user, &outcome);
    let _ = state.repo.save(&user);

    Json(PullResponse { result: SerializedRarity::try_from(outcome).unwrap()})
}

#[tokio::main]
async fn main() {
    let conn = Connection::open("./userdata.sql").unwrap();
    let _ = conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
        [],
    );


    let repo = SqliteRepo {db: conn.into() };
    let state = AppState { repo: repo.into() };
    let app = Router::new()
        .route("/pull", post(handle_pull))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
