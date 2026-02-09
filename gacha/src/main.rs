use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use chrono::{DateTime, Utc};
use gacha_protocol::{self, PityCtx, Rarities, roll};
use rand::{Rng, rng};
use rusqlite::Error as rusqError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Error as serdeError;
use serde_json::{self};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

#[derive(thiserror::Error, Debug)]
enum PersistenceError {
    #[error("Serialization of User Error")]
    JsonError(#[from] serdeError),
    #[error("Sqlite user loading error!")]
    DBError(#[from] rusqError),
    #[error("User Not Found")]
    NotFound,
    #[error("Invalid ID, Unable to construct voucher")]
    InvalidIDVoucher,
}
struct SqliteRepo {
    db: Mutex<Connection>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserId(String);
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Voucher {
    id: u64,
    name: String,
    cost: u64,
    description: String,
}
impl Voucher {
    fn by_id(id: u64) -> Result<Self, PersistenceError> {
        match id {
            999 => Ok(Self::mythic_week()),
            1 => Ok(Self::off_day()),
            10 => Ok(Self::gaming()),
            9 => Ok(Self::slip_gacha()),
            2 => Ok(Self::coffee()),

            _ => Err(PersistenceError::InvalidIDVoucher),
        }
    }
    fn new(id: u64, name: String, cost: u64, desc: String) -> Self {
        Self {
            id,
            name,
            cost,
            description: desc,
        }
    }
    fn mythic_week() -> Self {
        Self {
            id: 999,
            cost: 800000,
            name: "Mythic Week Off".to_string(),
            description: "Get a full week off".to_string(),
        }
    }
    fn off_day() -> Self {
        Self {
            id: 1,
            cost: 24000,
            name: String::from("Full Day Off"),
            description: String::from("Get one full day off"),
        }
    }
    fn gaming() -> Self {
        Self {
            id: 10,
            cost: 750,
            name: String::from("Gaming Session (3H)"),
            description: String::from("4 Hour Gaming Session"),
        }
    }
    fn slip_gacha() -> Self {
        Self {
            id: 9,
            cost: 450,
            name: String::from("Gacha Slip (8H)"),
            description: String::from(
                "Permission Slip for Working on the Gacha Machine for 8 Hours",
            ),
        }
    }
    fn coffee() -> Self {
        Self {
            id: 2,
            cost: 250,
            name: String::from("Coffee 250ml"),
            description: String::from("Get 1 cup of Coffee (Premium)"),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
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
    flux: i128,
    has_slip: bool,
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
            astrai: 10,
            flux: 500,
            has_slip: false,
            email: None,
            active_timer: false,
            timer: None,
            a_pity: 0,
            s_pity: 0,
            sss_pity: 0,
            total_pulls: 0,
            username: "Axol".to_string(),
            vouchers: Some(Vec::<Voucher>::new()),
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
    category: Category,
    reward: Option<u64>,
}
#[derive(Serialize, Deserialize, Clone)]
struct Timer {
    id: String,
    owner: UserId,
    started: DateTime<Utc>,
    ended: Option<i64>,
    category: Category,
}
impl TimerResponse {
    fn new(status: String, cat: Category) -> Self {
        Self {
            status,
            category: cat,
            reward: None,
        }
    }
    fn already_exists(cat: Category) -> Self {
        Self {
            status: "Timer Already Active".to_string(),
            category: cat,
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
    let mut vouchers: Vec<Voucher> = user.vouchers.clone().unwrap();
    match outcome {
        Rarities::MythicSSS => {
            user.sss_pity = 0;
            user.s_pity += 1;
            user.a_pity += 1;
            user.flux += 2400;

            let mut rng = rng();
            let roll: u8 = rng.random();

            if roll < 128 || user.has_slip {
                vouchers.push(Voucher::mythic_week());
            } else {
                vouchers.push(Voucher::gaming());
                vouchers.push(Voucher::off_day());
                user.has_slip = true;
            }
        }
        Rarities::S => {
            user.s_pity = 0;
            user.sss_pity += 1;
            user.a_pity += 1;
            user.flux += 400;
        }
        Rarities::A => {
            user.a_pity = 0;
            user.sss_pity += 1;
            user.s_pity += 1;

            user.flux += 100;
        }
        Rarities::B => {
            user.sss_pity += 1;
            user.s_pity += 1;
            user.a_pity += 1;

            user.flux += 5;
        }
    }

    user.vouchers = Some(vouchers);

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
    println!("Pulled!");
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
        return Ok(Json(TimerResponse::already_exists(
            user.timer.unwrap().category,
        )));
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
        status: "Timing...".to_string(),
        category: user.timer.unwrap().category,
        reward: None,
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
        return Ok(Json(TimerResponse::new(
            "No Active Timers".to_string(),
            Category::S_Node,
        )));
    }

    let timer: Timer = user.timer.unwrap();
    let reward = Some(resolve_timer(timer.clone()));
    user.active_timer = false;
    user.timer = None;
    user.astrum += reward.unwrap();
    let _ = state.repo.save(&user);

    Ok(Json(TimerResponse {
        status: format!("Timer Stopped"),
        category: timer.category,
        reward,
    }))
}
fn resolve_timer(timer: Timer) -> u64 {
    let duration = Utc::now() - timer.started;
    let hours = duration.num_hours();
    let minutes = duration.num_minutes();
    let mut reward: i64 = 0;

    if hours < 1 && minutes < 30 {
        return 0;
    }

    match timer.category {
        Category::S_Node => {
            reward = hours * 160;
        }
        Category::A_Node => {
            reward = ((minutes / 30) * 120) as i64;
        }
        Category::B_Node => {
            reward = ((minutes / 15) * 60) as i64;
        }
    }

    if reward < 1 {
        return 0;
    } else {
        return reward as u64;
    }
}

#[derive(Deserialize, Clone)]
struct VoucherRequest {
    userid: String,
    request_all: bool,
    filter_by_id: u64,
}
#[derive(Deserialize, Clone)]
struct PurchaseRequest {
    userid: String,
    amount: u8,
    id: u64,
}
async fn get_user_vouchers(
    State(state): State<AppState>,
    Json(req): Json<VoucherRequest>,
) -> Result<Json<Vec<Voucher>>, StatusCode> {
    let user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if req.request_all && req.filter_by_id == 0 {
        return Ok(Json(user.vouchers.unwrap()));
    }
    if req.request_all && req.filter_by_id > 0 {
        let vouchers_mapped: Vec<Voucher> = user
            .vouchers
            .unwrap()
            .into_iter()
            .filter(|e| e.id == req.filter_by_id)
            .collect();
        return Ok(Json(vouchers_mapped));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn purchase(
    State(state): State<AppState>,
    Json(req): Json<PurchaseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let req_voucher = Voucher::by_id(req.id).map_err(|_| StatusCode::NOT_FOUND)?;

    if user.flux < (req_voucher.cost * req.amount as u64) as i128 || req.amount < 1 {
        return Err(StatusCode::FORBIDDEN);
    }
    user.flux -= (req_voucher.cost * req.amount as u64) as i128;

    let mut vec = user.vouchers.unwrap();
    vec.push(req_voucher.clone());
    user.vouchers = Some(vec);

    let _ = state.repo.save(&user);
    Ok(Json(serde_json::json!({
        "result": format!(
        "Successfully Purchased Item: {}",
        &req_voucher.name),
    })))
}
#[derive(Deserialize)]
struct InfoRequest {
    userid: String,
}
async fn get_user_info(
    State(state): State<AppState>,
    Json(req): Json<InfoRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(serde_json::json!(
        {
            "astrum": user.astrum,
            "astrai": user.astrai,
            "flux": user.flux,
        }
    )))
}

#[tokio::main]
async fn main() {
    let repo = SqliteRepo::new("userdata.sql");
    let state = AppState { repo: repo.into() };

    let app = Router::new()
        .route("/start_timer", post(start_timer))
        .route("/pull", post(handle_pull))
        .route("/stop_timer", post(stop_timer))
        .route("/get_user_vouchers", post(get_user_vouchers))
        .route("/purchase", post(purchase))
        .route("/user_funds_info", post(get_user_info))
        .layer(CorsLayer::permissive())
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
