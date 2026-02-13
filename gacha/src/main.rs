use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::post};
use chrono::{Date, DateTime, Datelike, Duration, TimeZone, Utc};
use gacha_protocol::{self, PityCtx, Rarities, roll};
use rand::{Rng, rng};
use rusqlite::Error as rusqError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Error as serdeError;
use serde_json::{self};
use std::ops::Index;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

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
    uuid: Uuid,
    name: String,
    cost: u64,
    description: String,
}
impl Voucher {
    fn by_id(id: u64, userid: &str, state: Option<&AppState>) -> Result<Self, PersistenceError> {
        match id {
            999 => Ok(Self::mythic_week()),
            1 => Ok(Self::off_day()),
            2 => Ok(Self::gaming()),
            3 => Ok(Self::slip_gacha()),
            4 => Ok(Self::coffee()),
            5 => Ok(Self::coffee_standard()),
            24 => Ok(Self::japanese()),
            _ => {
                if userid.is_empty() {
                    return Err(PersistenceError::InvalidIDVoucher);
                }
                match state {
                    None => {
                        return Err(PersistenceError::InvalidIDVoucher);
                    }
                    _ => {}
                }

                let conn = state
                    .unwrap()
                    .repo
                    .db
                    .lock()
                    .map_err(|_| PersistenceError::DBError(rusqError::QueryReturnedNoRows))?;

                let mut stmt = conn
                    .prepare("SELECT json_extract(data, '$.templates) FROM users WHERE id = ?")?;

                let templates_json: String = stmt
                    .query_row(params![userid], |row| row.get(0))
                    .map_err(|_| PersistenceError::DBError(rusqError::QueryReturnedNoRows))?;

                let mut templates: Vec<Voucher> = serde_json::from_str(&templates_json)?;
                templates.retain(|f| f.id == id);
                let voucher = templates.index(0).clone();
                Ok(voucher)
            }
        }
    }
    fn new(id: u64, uuid: Uuid, name: String, cost: u64, desc: String) -> Self {
        Self {
            id,
            uuid,
            name,
            cost,
            description: desc,
        }
    }
    fn get_templates() -> Vec<Self> {
        let mut vec: Vec<Self> = Vec::new();
        let codes: [u16; 7] = [1, 2, 3, 4, 5, 24, 999];

        for x in codes.into_iter() {
            vec.push(Voucher::by_id(x as u64, "", None).unwrap());
        }
        vec
    }
    fn from_req_voucher(voucher: ReqVoucher) -> Self {
        Voucher::new(
            voucher.id,
            uuid::Uuid::now_v7(),
            voucher.name,
            voucher.cost,
            voucher.description,
        )
    }
    fn mythic_week() -> Self {
        Self {
            id: 999,
            uuid: uuid::Uuid::now_v7(),
            cost: 180000,
            name: "Mythic Week Off".to_string(),
            description: "Get a full week off".to_string(),
        }
    }
    fn off_day() -> Self {
        Self {
            id: 1,
            uuid: uuid::Uuid::now_v7(),
            cost: 1800,
            name: String::from("Full Day Off"),
            description: String::from("Get one full day off"),
        }
    }
    fn gaming() -> Self {
        Self {
            id: 2,
            uuid: uuid::Uuid::now_v7(),
            cost: 225,
            name: String::from("Gaming Session (2H)"),
            description: String::from("2 Hour Gaming Session"),
        }
    }
    fn slip_gacha() -> Self {
        Self {
            id: 3,
            uuid: uuid::Uuid::now_v7(),
            cost: 120,
            name: String::from("Gacha Slip (2H)"),
            description: String::from(
                "Permission Slip for Working on the Gacha Machine for 2 Hours",
            ),
        }
    }
    fn coffee() -> Self {
        Self {
            id: 4,
            uuid: uuid::Uuid::now_v7(),
            cost: 150,
            name: String::from("Coffee (Premium)"),
            description: String::from("Get 1 cup of Premium Coffee (250ml)"),
        }
    }
    fn coffee_standard() -> Self {
        Self {
            id: 5,
            uuid: uuid::Uuid::now_v7(),
            cost: 75,
            name: String::from("Coffee (Standard)"),
            description: String::from("Get 1 cup of Coffee Standard (250ml)"),
        }
    }
    fn japanese() -> Self {
        Self {
            id: 24,
            uuid: uuid::Uuid::now_v7(),
            cost: 120,
            name: String::from("Japanese Studies (3H)"),
            description: String::from("Slip to study Japanese for 3 Hours"),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
enum Category {
    SNode,
    ANode,
    BNode,
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
    dailies: Vec<Daily>,
    vouchers: Vec<Voucher>,
    templates: Vec<Voucher>,
    active_timer: bool,
    timer: Option<Timer>,
    sss_pity: u16,
    s_pity: u16,
    a_pity: u16,
    total_pulls: u128,
    total_flux_aq: u128,
    total_astrum_aq: u128,
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
        let init = false;
        let user = User {
            id: UserId("axol999".to_string()),
            astrum: 1600,
            astrai: 15,
            flux: 500,
            has_slip: false,
            email: None,
            active_timer: false,
            timer: None,
            a_pity: 0,
            s_pity: 0,
            sss_pity: 0,
            dailies: Daily::init(),
            total_pulls: 0,
            total_flux_aq: 0,
            total_astrum_aq: 0,
            username: "Axol".to_string(),
            vouchers: Vec::<Voucher>::new(),
            templates: Voucher::get_templates(),
        };
        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
            [],
        );

        if init {
            let _ = conn.execute(
                "INSERT OR REPLACE INTO users (id, data) Values(?1, ?2)",
                params![user.id.0, serde_json::to_string(&user).unwrap()],
            );
        }

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

fn load_user(userid: String, state: &AppState) -> Result<User, StatusCode> {
    let user = state
        .repo
        .load(UserId(userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(user)
}

fn quick_roll() -> u8 {
    let mut rng = rng();
    let roll: u8 = rng.random();
    roll
}
fn apply_outcome(user: &mut User, outcome: &Rarities) {
    let mut vouchers: Vec<Voucher> = user.vouchers.clone();
    match outcome {
        Rarities::MythicSSS => {
            user.sss_pity = 0;
            user.s_pity += 1;
            user.a_pity += 1;
            user.flux += 2400;
            user.total_flux_aq += 2400;

            if quick_roll() < 128 || user.has_slip {
                vouchers.push(Voucher::mythic_week());
            } else {
                vouchers.push(Voucher::gaming());
                vouchers.push(Voucher::gaming());
                vouchers.push(Voucher::off_day());

                for _x in 0..3 {
                    vouchers.push(Voucher::coffee());
                    vouchers.push(Voucher::japanese());
                }
                user.has_slip = true;
            }
        }
        Rarities::S => {
            user.s_pity = 0;
            user.sss_pity += 1;
            user.a_pity += 1;
            user.flux += 240;
            user.total_flux_aq += 240;

            vouchers.push(Voucher::new(
                24,
                uuid::Uuid::now_v7(),
                "Japanese [3H]".into(),
                40,
                "Study Japanese for 3 Hours".into(),
            ));

            if quick_roll() < 85 {
                vouchers.push(Voucher::new(
                    9,
                    uuid::Uuid::now_v7(),
                    "Gacha Slip [2H]".into(),
                    30,
                    "Permission Slip to Work on LifeGacha for 2 Hours".into(),
                ));
            }
            if quick_roll() < 85 {
                vouchers.push(Voucher::coffee());
            }

            if quick_roll() < 16 {
                vouchers.push(Voucher::gaming());
            }
        }
        Rarities::A => {
            user.a_pity = 0;
            user.sss_pity += 1;
            user.s_pity += 1;

            user.flux += 75;
            user.total_flux_aq += 75;

            vouchers.push(Voucher::new(
                0,
                uuid::Uuid::now_v7(),
                "15 Minute Break".into(),
                20,
                "Take a 15 minute breather break".into(),
            ));
        }
        Rarities::B => {
            user.sss_pity += 1;
            user.s_pity += 1;
            user.a_pity += 1;

            user.flux += 5;
            user.total_flux_aq += 5;

            if quick_roll() == 24 {
                vouchers.push(Voucher::gaming());
            }
        }
    }

    user.vouchers = vouchers;

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
    let mut user = load_user(req.userid, &state)?;

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
    let mut user = load_user(req.userid, &state)?;

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
    let mut user = load_user(req.userid, &state)?;

    if !user.active_timer {
        return Ok(Json(TimerResponse::new(
            "No Active Timers".to_string(),
            Category::SNode,
        )));
    }

    let timer: Timer = user.timer.unwrap();
    let reward = Some(resolve_timer(timer.clone()));

    user.active_timer = false;
    user.timer = None;
    user.astrum += reward.unwrap();
    user.total_astrum_aq += reward.unwrap() as u128;

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

    if hours < 1 && minutes < 1 {
        return 0;
    }

    match timer.category {
        Category::SNode => reward = ((minutes as f64 / 60.0) * 350.0) as i64,
        Category::ANode => {
            reward = ((minutes as f64 / 30.0) as f64 * 80.0) as i64;
        }
        Category::BNode => {
            reward = ((minutes as f64 / 10.0) as f64 * 20.0) as i64;
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
    store: bool,
}
async fn get_user_vouchers(
    State(state): State<AppState>,
    Json(req): Json<VoucherRequest>,
) -> Result<Json<Vec<Voucher>>, StatusCode> {
    let user = state
        .repo
        .load(UserId(req.userid))
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if req.store {
        return Ok(Json(user.templates));
    }

    if req.request_all && req.filter_by_id == 0 {
        return Ok(Json(user.vouchers));
    }
    if req.request_all && req.filter_by_id > 0 {
        let vouchers_mapped: Vec<Voucher> = user
            .vouchers
            .into_iter()
            .filter(|e| e.id == req.filter_by_id)
            .collect();
        return Ok(Json(vouchers_mapped));
    }
    Err(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct ReqVoucher {
    id: u64,
    cost: u64,
    name: String,
    description: String,
}
#[derive(Deserialize)]
struct CreateRequest {
    userid: String,
    voucher: ReqVoucher,
}
async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut user = load_user(req.userid, &state)?;
    let voucher = req.voucher;
    if voucher.cost < 1 || voucher.name.is_empty() {
        return Err(StatusCode::LENGTH_REQUIRED);
    }

    user.templates.push(Voucher::from_req_voucher(voucher));
    let _ = state.repo.save(&user);
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Clone)]
struct PurchaseRequest {
    userid: String,
    amount: u8,
    id: u64,
}
async fn purchase(
    State(state): State<AppState>,
    Json(req): Json<PurchaseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut user = load_user(req.userid.clone(), &state)?;

    let req_voucher =
        Voucher::by_id(req.id, &req.userid, Some(&state)).map_err(|_| StatusCode::NOT_FOUND)?;

    if user.flux < (req_voucher.cost * req.amount as u64) as i128 || req.amount < 1 {
        return Err(StatusCode::FORBIDDEN);
    }
    user.flux -= (req_voucher.cost * req.amount as u64) as i128;

    let mut vec = user.vouchers;
    vec.push(req_voucher.clone());
    user.vouchers = vec;

    let _ = state.repo.save(&user);
    Ok(Json(serde_json::json!({
        "result": format!(
        "Purchased Item: {}",
        &req_voucher.name),
    })))
}

#[derive(Deserialize)]
struct ConsumeRequest {
    userid: String,
    uuid: Uuid,
}
async fn consume(
    State(state): State<AppState>,
    Json(req): Json<ConsumeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut user = load_user(req.userid.clone(), &state)?;
    let vec: Vec<Voucher>;

    vec = user
        .vouchers
        .into_iter()
        .filter(|f| f.uuid != req.uuid)
        .collect();
    user.vouchers = vec;

    let _ = state.repo.save(&user);
    Ok(Json(serde_json::json!({
        "status": "Item Consumed",
    })))
}

#[derive(Deserialize, Serialize, Clone)]
struct Daily {
    id: u8,
    claimable: bool,
    claimed: bool,
    last_claimed: i64,
}
impl Daily {
    fn init() -> Vec<Self> {
        let mut vec: Vec<Self> = Vec::new();
        for i in 0..4 {
            vec.push(Daily {
                id: i as u8,
                claimable: true,
                claimed: false,
                last_claimed: 0 as i64,
            });
        }
        vec
    }
}
#[derive(Deserialize)]
struct DailiesReq {
    userid: String,
    info: bool,
    id: u8,
}
#[derive(Serialize)]
struct DailiesResp {
    dailies: Vec<Daily>,
}
async fn dailies(
    State(state): State<AppState>,
    Json(req): Json<DailiesReq>,
) -> Result<Json<DailiesResp>, StatusCode> {
    let mut user = load_user(req.userid, &state)?;

    let now = Utc::now();
    let mut cycle_start = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 4, 0, 0)
        .unwrap();

    if now < cycle_start {
        cycle_start -= Duration::days(1);
    }

    if req.info {
        for daily in user.dailies.iter_mut() {
            if daily.last_claimed >= cycle_start.timestamp() {
                daily.claimable = false;
            } else {
                daily.claimable = true;
                daily.claimed = false;
            }
        }

        let _ = state.repo.save(&user);
        return Ok(Json(DailiesResp {
            dailies: user.dailies.clone(),
        }));
    }
    if req.id < 4 && !req.info {
        user.dailies[req.id as usize].last_claimed = Utc::now().timestamp();
        user.dailies[req.id as usize].claimed = true;

        let _ = state.repo.save(&user);
    }

    Err(StatusCode::FORBIDDEN)
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
        .route("/consume", post(consume))
        .route("/create", post(create))
        .route("/dailies", post(dailies))
        .layer(CorsLayer::permissive())
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("11.0.0.2:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
