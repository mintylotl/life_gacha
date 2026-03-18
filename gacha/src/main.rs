use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Json, Router, routing::get, routing::post};
use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use rand::{Rng, rng};
use rusqlite::Error as rusqError;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Error as serdeError;
use serde_json::{self};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::task::{AbortHandle, JoinHandle};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

// Custom Types1
use BarType::*;
use Coeff::*;
use gacha_protocol::{self, PityCtx, Rarities, roll};
//use std::time::Duration as Duration_Time;

#[derive(thiserror::Error, Debug)]
enum PersistenceError {
    #[error("Serialization of User Error")]
    JsonError(#[from] serdeError),
    #[error("Sqlite user loading error!")]
    DBError(#[from] rusqError),
    #[error("User Not Found")]
    NotFound,
    //#[error("Invalid ID, Unable to construct voucher")]
    //InvalidIDVoucher,
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
    #[serde(default)]
    dur: f64,
    new: bool,
    description: String,
    coeff: Coeff,
}
impl Voucher {
    fn minutes(&self) -> f64 {
        self.dur.round()
    }
    fn hours(&self) -> f64 {
        ((self.dur / 60.0) * 100.0).round() / 100.0
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
enum Coeff {
    PureC(f64, u32),
    FunG(f64, u32),
    G(f64, u32),
    Expansion(f64, u32),
    Base(f64, u32),
    Maintenance(f64, u32),
    System(f64, u32),
}

impl Coeff {
    fn get_val(&self) -> u64 {
        if let PureC(coeff, base_cost) = self {
            return (*coeff * *base_cost as f64) as u64;
        }
        if let FunG(coeff, base_cost) = self {
            return (*coeff * *base_cost as f64) as u64;
        }
        if let G(coeff, base_cost) = self {
            return (*coeff * *base_cost as f64) as u64;
        }
        if let Expansion(coeff, base_cost) = self {
            return (*coeff * *base_cost as f64) as u64;
        }
        if let Base(coeff, base_const) = self {
            return (*coeff * *base_const as f64) as u64;
        }
        if let Maintenance(coeff, base_const) = self {
            return (*coeff * *base_const as f64) as u64;
        }
        if let System(coeff, base_const) = self {
            return (*coeff * *base_const as f64) as u64;
        }

        return 0;
    }
    fn pure_c() -> Self {
        Self::PureC(1.72, 120)
    }
    fn fun_g() -> Self {
        Self::FunG(1.48, 120)
    }
    fn g() -> Self {
        Self::G(1.24, 120)
    }
    fn exp() -> Self {
        Self::Expansion(1.12, 120)
    }
    fn base() -> Self {
        Self::Base(1.0, 120)
    }
    fn maint() -> Self {
        Self::Maintenance(1.0, 120)
    }
    fn system() -> Self {
        Self::System(1.1, 120)
    }
}

impl From<String> for Coeff {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl From<&str> for Coeff {
    fn from(value: &str) -> Self {
        match value {
            "g" => Self::g(),
            "fun_g" => Self::fun_g(),
            "exp" => Self::exp(),
            "pure_c" => Self::pure_c(),
            "maint" => Self::maint(),
            "base" => Self::base(),
            _ => Self::system(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct Bar {
    id: u8,
    locked: bool,
    is_timing: bool,
    overdrive: bool,
    c: f64,
    s: f64,
    smax: f64,
    tmax: f64,
    tbase: f64,
    s_reduction: f64,
    overdrive_val: f64,
}
enum BarType {
    Stab,
    Exp,
    Maint,
    Leisure,
    Meta,
    Idle,
    Sys,
}
impl From<Voucher> for BarType {
    fn from(value: Voucher) -> Self {
        match value.coeff {
            Base(..) => Idle,
            Expansion(..) => Exp,
            G(..) => Stab,
            FunG(..) => Meta,
            PureC(..) => Leisure,
            Maintenance(..) => Maint,
            System(..) => Sys,
        }
    }
}
impl BarType {
    fn get_fx_pool(voucher: Voucher, state: AppState) -> Result<JoinHandle<f64>, u8> {
        let variant: usize = match BarType::from(voucher.clone()) {
            Stab => 0,
            Exp => 1,
            Maint => 2,
            Leisure => 3,
            Meta => 4,
            Idle => 5,
            Sys => 255,
        };

        match variant {
            255 => Err(1),
            _ => {
                if let Some(timer) = state.timer.lock().unwrap().take() {
                    timer.abort();
                }
                let mut bars = state.bars.lock().unwrap();
                let bar = bars.get(variant).unwrap().clone();
                let mut timer = state.timer.lock().unwrap();
                let initial_s = bar.s;
                let id = bar.id;

                let cost_hours = voucher.minutes();

                if bar.smax - bar.s < cost_hours {
                    return Err(2);
                }

                bars.iter_mut().for_each(|f| {
                    f.is_timing = false;
                });
                bars.iter_mut().for_each(|f| {
                    if f.id == id {
                        f.smax = cost_hours;
                        f.s = 0.0;
                    }
                });
                let handle_init = bar.start_timer(state.clone());
                *timer = Some(handle_init.clone());

                let handle = handle_init;
                let state_i = state.clone();
                Ok(tokio::spawn(async move {
                    let mut interval =
                        tokio::time::interval(Duration::seconds(2).to_std().unwrap());
                    let init_s = initial_s;
                    let result = loop {
                        interval.tick().await;
                        if handle.is_finished() {
                            let mut bars = state_i.bars.lock().unwrap();
                            let bar = bars.get_mut(id as usize).unwrap();
                            let result = bar.s;

                            bar.smax = Bar::by_id(id).smax;
                            bar.s += init_s;

                            break result;
                        }
                    };
                    result
                }))
            }
        }
    }
}
impl Bar {
    fn _vec() -> Vec<Bar> {
        let mut vec = Vec::<Bar>::new();
        for x in 0..6 {
            vec.push(Bar::by_id(x));
        }

        vec
    }
    fn by_id(id: u8) -> Self {
        let mut bar = Self {
            id: id,
            c: 0.0,
            is_timing: false,
            overdrive: false,
            s: 0.0,
            smax: 0.0,
            locked: false,
            tbase: 0.0,
            tmax: 0.0,
            s_reduction: 0.0,
            overdrive_val: 0.0,
        };
        match id {
            0 => {
                bar.c = 1.5;
                bar.smax = 240.0;
                bar.tmax = 360.0;
                bar.overdrive_val = (0.24 * bar.smax) + 15.0;
                bar
            }
            1 => {
                bar.c = 1.2;
                bar.smax = 210.0;
                bar.tmax = 360.0;
                bar.overdrive_val = (0.24 * bar.smax) + 15.0;
                bar
            }
            2 => {
                bar.c = 1.0;
                bar.smax = 90.0;
                bar.tmax = 480.0;
                bar.overdrive_val = (0.24 * bar.smax) + 15.0;
                bar
            }
            3 => {
                bar.c = 0.7;
                bar.smax = 150.0;
                bar.tmax = 600.0;
                bar.overdrive_val = (0.24 * bar.smax) + 15.0;
                bar
            }
            4 => {
                bar.c = 1.1;
                bar.smax = 90.0;
                bar.tmax = 480.0;
                bar.overdrive_val = (0.24 * bar.smax) + 15.0;
                bar
            }
            5 => {
                bar.c = 0.0;
                bar.smax = 180.0;
                bar.tmax = 10000.0;
                bar.overdrive_val = (0.24 * bar.smax) + 15.0;
                bar
            }
            _ => bar,
        }
    }

    fn start_idle(&self, state: AppState) -> AbortHandle {
        let id = self.id;
        tokio::spawn(async move {
            let state = state;
            let mut interval = tokio::time::interval(Duration::minutes(1).to_std().unwrap());
            let mut eat = false;
            interval.tick().await;

            {
                let mut bars = state.bars.lock().unwrap();
                let bar = bars.get_mut(id as usize).unwrap();
                bar.is_timing = true;
            }
            loop {
                interval.tick().await;
                let mut bars = state.bars.lock().unwrap();
                let bar = bars.get_mut(id as usize).unwrap();
                bar.s += 1.0;

                if bar.s >= bar.smax {
                    if !bar.overdrive {
                        bar.overdrive = true;
                        bar.s -= bar.overdrive_val
                    } else {
                        bar.locked = true;
                        bar.is_timing = false;
                        eat = true;
                    }
                }

                if eat {
                    let handle = bars.get_mut(3).unwrap().start_timer(state.clone());
                    let mut timer = state.timer.lock().unwrap();
                    *timer = Some(handle);

                    break;
                }
            }
        })
        .abort_handle()
    }

    fn start_timer(&self, state: AppState) -> AbortHandle {
        let state_i = state.clone();
        let id = self.id;

        tokio::spawn(async move {
            let time = 15.0;
            let state = state_i;
            let id = id;
            let mut interval = tokio::time::interval(Duration::seconds(5).to_std().unwrap());
            interval.tick().await;

            {
                let mut bars = state.bars.lock().unwrap();

                if bars.iter().filter(|f| !f.locked).count() == 1 {
                    bars.iter_mut().filter(|f| !f.locked).for_each(|bar| {
                        bar.reduce_s(bar.overdrive_val - bar.s_reduction, 1.0);
                    });
                }

                let bar = bars.get_mut(id as usize).unwrap();

                if bar.locked {
                    return;
                }
                if bar.tbase > bar.smax && !bar.overdrive {
                    bar.overdrive = true;
                }

                bar.is_timing = true;
            }

            loop {
                {
                    let mut bars = state.bars.lock().unwrap();

                    let bar = bars.get_mut(id as usize).unwrap();
                    if bar.s >= bar.smax {
                        if bar.overdrive && bar.s_reduction >= bar.overdrive_val {
                            bar.locked = true;
                            bar.tbase = bar.tmax;
                        }
                        bar.s = bar.smax;
                        bar.smax = Bar::by_id(id).smax;
                        bar.is_timing = false;

                        let mut timer = state.timer.lock().unwrap();
                        *timer = Some(Bar::by_id(5).start_idle(state.clone()));

                        break;
                    }
                }
                interval.tick().await;

                let mut bars = state.bars.lock().unwrap();
                let bar = bars.get_mut(id as usize).unwrap();
                let c = bar.c;
                let smax = if bar.overdrive {
                    bar.smax + bar.overdrive_val
                } else {
                    bar.smax
                };

                bar.s = smax.min(bar.s + time);
                bar.tbase = (smax + 1.0).min(bar.tbase + time);

                bars.iter_mut().for_each(|bar_f| {
                    if bar_f.id != id {
                        bar_f.reduce_tbase(time, c);
                        bar_f.reduce_s(time, c);
                    }

                    if bar_f.id == id {
                        if bar_f.tbase > bar_f.smax {
                            bar_f.overdrive = true;
                        }
                    }
                });
            }
        })
        .abort_handle()
    }

    fn reset(&mut self) {
        self.locked = false;
        self.s = 0.0;
        self.s_reduction = 0.0;
        self.overdrive = false;
        self.tbase = 0.0;
    }
    fn reduce_tbase(&mut self, time: f64, c: f64) {
        if self.locked {
            self.tbase -= (time * c).max(0.0);

            if self.tbase <= 0.0 {
                self.reset();
            }
        }
    }
    fn reduce_s(&mut self, time: f64, c: f64) {
        if self.locked {
            let percentage = self.tbase.max(0.1) / self.tmax;
            let s = percentage * self.smax;
            self.s = s.max(0.0);
        }
        if self.s_reduction < self.overdrive_val {
            if !(self.s - (time * c) < 0.0) {
                self.s -= time * c;
                self.s_reduction += c * time;
            } else {
                self.s = 0.0;
            }
        }
    }
}

fn round_h(val: f64) -> f64 {
    ((val / 60.0) * 100.0).round() / 100.0
}
impl Voucher {
    fn by_id(id: u64, user: &User) -> Result<Self, PersistenceError> {
        match id {
            999 => Ok(Self::mythic_week()),
            1 => Ok(Self::off_day()),
            4 => Ok(Self::coffee()),
            _ => {
                let mut vouchers = user.templates.clone();
                vouchers.retain(|v| v.id == id);

                let mut voucher = vouchers.get(0).unwrap().clone();
                voucher.uuid = uuid::Uuid::now_v7();
                Ok(voucher)
            }
        }
    }
    fn new(
        id: u64,
        uuid: Uuid,
        name: String,
        cost: u64,
        dur: f64,
        new: bool,
        desc: String,
        coeff: Coeff,
    ) -> Self {
        Self {
            id,
            uuid,
            name,
            cost,
            dur,
            new,
            description: desc,
            coeff,
        }
    }
    fn _get_templates(user: &User) -> Vec<Self> {
        let mut vec: Vec<Self> = Vec::new();
        let codes: [u16; 3] = [1, 4, 999];

        for x in codes.into_iter() {
            vec.push(Voucher::by_id(x as u64, user).unwrap());
        }
        vec
    }
    fn from_req_voucher(user: &User, voucher: ReqVoucher) -> Self {
        let mut highest_found: u64 = 0;

        let highest_c = &mut highest_found;
        user.templates
            .iter()
            .map(|t| {
                if t.id > *highest_c && t.id != 999 {
                    *highest_c = t.id;
                }
            })
            .count();

        let mut id = highest_found + 100;
        if id == 999 {
            id += 1;
        }

        let coeff: Coeff = voucher.coeff.into();
        //let cost = coeff.get_val() * voucher.dur as u64;
        let cost = 3;
        let name = format!("{}", voucher.name);
        let dur = voucher.dur * 60.0;
        let description = voucher.description;

        Voucher::new(
            id,
            uuid::Uuid::now_v7(),
            name,
            cost,
            dur,
            true,
            description,
            coeff,
        )
    }
    fn from_purchase_req(req: PurchaseRequest, state: &AppState) -> Self {
        let user = load_user(req.userid, state).unwrap().0;
        let mut req_voucher = Voucher::by_id(req.id, &user).unwrap();

        match req_voucher.id {
            1 | 4 | 999 => {
                req_voucher.dur = 1.0;
                return req_voucher;
            }
            _ => (),
        }

        req_voucher.name = format!("{} [{}H]", &req_voucher.name, req.dur.to_string());
        req_voucher.cost = 3;
        req_voucher.dur = (req.dur * 60.0).round();

        req_voucher
    }
    fn mythic_week() -> Self {
        Self {
            id: 999,
            uuid: uuid::Uuid::now_v7(),
            name: "Mythic Week Off".to_string(),
            cost: 35480,
            new: true,
            dur: 168.0 * 60.0,
            description: "Get a full week off".to_string(),
            coeff: Coeff::pure_c(),
        }
    }
    fn off_day() -> Self {
        Self {
            id: 1,
            uuid: uuid::Uuid::now_v7(),
            name: String::from("Full Day Off"),
            cost: 5720,
            dur: 24.0 * 60.0,
            new: true,
            description: String::from("Get one full day off"),
            coeff: Coeff::pure_c(),
        }
    }
    fn coffee() -> Self {
        Self {
            id: 4,
            uuid: uuid::Uuid::now_v7(),
            name: String::from("Coffee (Premium)"),
            cost: 150,
            dur: 0.0 * 60.0,
            new: true,
            description: String::from("Get 1 cup of Premium Coffee (250ml)"),
            coeff: Coeff::base(),
        }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Copy)]
enum Category {
    SNode,
    ANode,
    BNode,
}

#[derive(Serialize, Deserialize, Clone)]
struct ISRDO {
    uuid: uuid::Uuid,
    description: String,
    payout: u16,
}

trait UserRepo {
    fn load<'a>(
        &'a self,
        id: UserId,
    ) -> Result<(User, MutexGuard<'a, Connection>), PersistenceError>;
    fn save(&self, user: &User, conn: &Connection) -> Result<(), PersistenceError>;
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
    bars: Vec<Bar>,
    active_timer: bool,
    active_bar: u8,
    pause_drip: bool,
    timer: Option<Timer>,
    timeout_map: HashMap<u64, i64>,
    isrdos: Vec<ISRDO>,
    sss_pity: u16,
    s_pity: u16,
    a_pity: u16,
    total_pulls: u128,
    total_flux_aq: u128,
    total_astrum_aq: u128,
    todays_flux: (i64, u64),
}
impl UserRepo for SqliteRepo {
    fn load<'a>(
        &'a self,
        id: UserId,
    ) -> Result<(User, MutexGuard<'a, Connection>), PersistenceError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT data FROM users WHERE id = ?1")?;
        let user_json: String;
        let user: User;
        {
            user_json = stmt
                .query_row(params![id.0], |row| row.get(0))
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => PersistenceError::NotFound,
                    _ => PersistenceError::DBError(e),
                })?;

            user = serde_json::from_str(&user_json)?;
            drop(stmt);
        };
        Ok((user, conn))
    }
    fn save(&self, user: &User, conn: &Connection) -> Result<(), PersistenceError> {
        let user_bytes = serde_json::to_string_pretty(user)?;

        conn.execute(
            "INSERT OR REPLACE INTO users (id, data) VALUES (?1, ?2)",
            params![user.id.0, user_bytes],
        )?;
        Ok(())
    }
}
impl SqliteRepo {
    /*fn mark_all_vouchers_new(&self) -> Result<(), rusqlite::Error> {
        let conn = self.db.lock().unwrap();
        conn.execute(
            "UPDATE users
             SET data = json_set(
                 data,
                 '$.templates',
                 (SELECT json_group_array(json_set(value, '$.new', json('true')))
                  FROM json_each(users.data, '$.vouchers'))
             )",
            [],
        )?;
        Ok(())
    }*/
    fn new(path: &str) -> Self {
        let conn = Connection::open(path).expect("Error opening DB!");
        //let init = false;
        /*let mut user = User {
            id: UserId("testing".to_string()),
            astrum: 1600,
            astrai: 30,
            flux: 500,
            has_slip: false,
            email: None,
            active_timer: false,
            timer: None,
            timeout_map: HashMap::new(),
            a_pity: 0,
            s_pity: 0,
            sss_pity: 0,
            dailies: Daily::init(),
            todays_flux: (0, 0),
            total_pulls: 0,
            total_flux_aq: 0,
            total_astrum_aq: 0,
            username: "Test User".to_string(),
            vouchers: Vec::<Voucher>::new(),
            templates: Vec::<Voucher>::new(),
            isrdos: Vec::<ISRDO>::new(),
            pause_drip: false,
        };*/
        //user.templates = Voucher::get_templates(&user);

        let _ = conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
            [],
        );

        //if init {
        //    let _ = conn.execute(
        //       "INSERT OR REPLACE INTO users (id, data) Values(?1, ?2)",
        //        params![user.id.0, serde_json::to_string(&user).unwrap()],
        //    );
        //}

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

#[derive(Serialize, Deserialize, Clone)]
struct Timer {
    id: String,
    owner: UserId,
    started: DateTime<Utc>,
    ended: Option<DateTime<Utc>>,
    category: Category,
}

#[derive(Deserialize)]
struct PullRequest {
    userid: String,
}
#[derive(Serialize)]
struct PullResponse {
    result: SerializedRarity,
    vouchers: u8,
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
    timer: Arc<Mutex<Option<AbortHandle>>>,
    bars: Arc<Mutex<Vec<Bar>>>,
}

fn load_user(
    userid: String,
    state: &AppState,
) -> Result<(User, MutexGuard<'_, Connection>), StatusCode> {
    Ok(state
        .repo
        .load(UserId(userid))
        .map_err(|_| StatusCode::NOT_FOUND)?)
}

fn save_user(
    user: &User,
    conn: &MutexGuard<'_, Connection>,
    state: &AppState,
) -> Result<(), StatusCode> {
    Ok(state
        .repo
        .save(user, conn)
        .map_err(|_| StatusCode::NOT_FOUND)?)
}

fn quick_roll() -> u8 {
    let mut rng = rng();
    let roll: u8 = rng.random();
    roll
}
fn apply_outcome(user: &mut User, outcome: &Rarities) -> u8 {
    let mut vouchers: Vec<Voucher> = user.vouchers.clone();
    let mut rw_vouchers: u8 = 0;

    match outcome {
        Rarities::MythicSSS => {
            user.sss_pity = 0;
            user.s_pity += 1;
            user.a_pity += 1;
            user.flux += 2400;
            user.total_flux_aq += 2400;

            if quick_roll() < 128 || user.has_slip {
                vouchers.push(Voucher::mythic_week());
                rw_vouchers += 1;
            } else {
                vouchers.push(Voucher::off_day());

                rw_vouchers += 1;
                user.has_slip = true;
            }
        }
        Rarities::S => {
            user.s_pity = 0;
            user.sss_pity += 1;
            user.a_pity += 1;
            user.flux += 240;
            user.total_flux_aq += 240;

            if quick_roll() < 64 {
                rw_vouchers += 1;
            } else {
                rw_vouchers += 1;
            }

            if quick_roll() < 16 {
                rw_vouchers += 1;
            }
            if quick_roll() < 16 {
                rw_vouchers += 1;
            }
        }
        Rarities::A => {
            user.a_pity = 0;
            user.sss_pity += 1;
            user.s_pity += 1;

            user.flux += 70;
            user.total_flux_aq += 70;

            if quick_roll() < 200 {
                vouchers.push(Voucher::new(
                    0,
                    uuid::Uuid::now_v7(),
                    "15 Minute Break".into(),
                    (Coeff::pure_c().get_val() as f64 * 0.25) as u64,
                    0.0,
                    true,
                    "Take a 15 minute breather break".into(),
                    Coeff::pure_c(),
                ));
                rw_vouchers += 1;
            }
        }
        Rarities::B => {
            user.sss_pity += 1;
            user.s_pity += 1;
            user.a_pity += 1;

            user.flux += 4;
            user.total_flux_aq += 4;

            if quick_roll() == 24 {
                vouchers.push(Voucher::off_day());
                rw_vouchers += 1;
            }
        }
    }

    user.vouchers = vouchers;

    user.total_pulls += 1;
    if user.astrai > 0 {
        user.astrai -= 1;
        return rw_vouchers;
    }
    user.astrum -= 160;
    rw_vouchers
}

async fn handle_pull(
    State(state): State<AppState>,
    Json(req): Json<PullRequest>,
) -> Result<Json<PullResponse>, StatusCode> {
    let (mut user, conn) = load_user(req.userid, &state)?;

    if user.astrai < 1 && user.astrum < 160 {
        return Ok(Json(PullResponse {
            result: SerializedRarity::NoTickets,
            vouchers: 0,
        }));
    }

    let pityctx = PityCtx::try_from(&user).unwrap();
    let outcome = roll(&pityctx);

    let reward = apply_outcome(&mut user, &outcome);
    let _ = state.repo.save(&user, &conn);

    Ok(Json(PullResponse {
        result: SerializedRarity::try_from(outcome).unwrap(),
        vouchers: reward,
    }))
}

/*
fn resolve_timer(timer: Timer) -> u64 {
    let reward: i64;
    let duration: Duration;

    if let Some(ended) = timer.ended {
        duration = ended - timer.started;
    } else {
        duration = Utc::now() - timer.started;
    }

    let hours = duration.num_hours();
    let minutes = duration.num_minutes();

    if hours < 1 && minutes < 1 {
        return 0;
    }

    match timer.category {
        Category::SNode => reward = ((minutes as f64 / 60.0) * 480.0) as i64,
        Category::ANode => {
            reward = ((minutes as f64 / 30.0) as f64 * 120.0) as i64;
        }
        Category::BNode => {
            reward = ((minutes as f64 / 10.0) as f64 * 25.0) as i64;
        }
    }

    if reward < 1 {
        return 0;
    } else {
        return reward as u64;
    }
}*/

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
    let (user, _conn) = load_user(req.userid, &state)?;
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
    name: String,
    dur: f64,
    coeff: String,
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
    let (mut user, conn) = load_user(req.userid, &state)?;
    let voucher = req.voucher;
    if voucher.dur < 0.1 || voucher.name.is_empty() {
        return Err(StatusCode::LENGTH_REQUIRED);
    }

    user.templates
        .push(Voucher::from_req_voucher(&user, voucher));
    let _ = state.repo.save(&user, &conn);
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize, Clone)]
struct PurchaseRequest {
    userid: String,
    amount: u8,
    id: u64,
    dur: f64,
}
async fn purchase(
    State(state): State<AppState>,
    Json(req): Json<PurchaseRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let req_voucher = Voucher::from_purchase_req(req.clone(), &state);

    let (mut user, conn) = load_user(req.userid.clone(), &state)?;
    let cost = req_voucher.cost * req.amount as u64 * req.dur as u64;

    if user.flux < cost as i128 || req.amount < 1 {
        return Err(StatusCode::FORBIDDEN);
    }

    decrease_flux(&mut user, cost as i128);

    user.vouchers.push(req_voucher.clone());
    let _ = state.repo.save(&user, &conn);

    Ok(Json(serde_json::json!({
        "name": &req_voucher.name,
        "result": format!(
        "Purchased Item: {}",
        &req_voucher.name),
    })))
}

#[derive(Deserialize)]
struct DeleteRequest {
    userid: String,
    uuid: Uuid,
    store: bool,
}
async fn delete_item(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (mut user, conn) = load_user(req.userid.clone(), &state)?;

    let template = user.templates.iter().find(|v| v.uuid == req.uuid);
    let voucher = user.vouchers.iter().find(|v| v.uuid == req.uuid);
    if voucher.is_none() && !req.store || template.is_none() && req.store {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut action = String::new();
    if req.store {
        user.templates.retain(|v| v.uuid != req.uuid);
        action += "Deleted";
    } else {
        action += "Refunded";
        if let Some(refund) = user.vouchers.iter().find(|v| v.uuid == req.uuid) {
            user.flux += refund.cost as i128;
            user.vouchers.retain(|v| v.uuid != req.uuid);
        }
    }
    let _ = state.repo.save(&user, &conn);

    Ok(Json(serde_json::json!({
        "status": format!("Item {}", action),
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
    let vec: Vec<Voucher>;
    let voucher_opt: Option<Voucher>;
    {
        let user = load_user(req.userid.clone(), &state)?.0;

        voucher_opt = user
            .vouchers
            .clone()
            .into_iter()
            .filter(|v| v.uuid == req.uuid)
            .next();
    }

    if let Some(voucher) = voucher_opt {
        let avail = BarType::get_fx_pool(voucher.clone(), state.clone());
        match avail {
            Ok(result) => {
                let username = req.userid.clone();
                let state_i = state.clone();

                tokio::spawn(async move {
                    let state = state_i;
                    let username = username;

                    loop {
                        tokio::time::sleep(Duration::seconds(5).to_std().unwrap()).await;
                        if result.is_finished() {
                            let new_res = result.await;
                            if new_res.is_ok() {
                                let time = new_res.unwrap();

                                let mut voucher = voucher;
                                let mut time_remain = voucher.minutes() - time;
                                let prefix: &str;

                                prefix = match time_remain {
                                    t if t < 60.0 => "M",
                                    _ => "H",
                                };

                                if time_remain <= 5.0 {
                                    break;
                                }
                                let hours_remain = round_h(time_remain);
                                let minutes_remain = time_remain;
                                time_remain = match prefix {
                                    "M" => time_remain.round(),
                                    _ => hours_remain,
                                };

                                let hours = voucher.hours();
                                let mut user = load_user(username.clone(), &state).unwrap().0;
                                voucher.name = Voucher::by_id(voucher.id, &user).unwrap().name
                                    + " ["
                                    + &time_remain.to_string()
                                    + prefix
                                    + "]";
                                voucher.dur = minutes_remain;

                                match time {
                                    0.0 => (),
                                    t if t < 5.0 => {
                                        user.vouchers.push(Voucher::from_purchase_req(
                                            PurchaseRequest {
                                                userid: get_username(),
                                                amount: 1,
                                                id: voucher.id,
                                                dur: hours,
                                            },
                                            &state,
                                        ));
                                    }
                                    _ => {
                                        user.vouchers.push(voucher);
                                    }
                                }

                                let conn = load_user(username, &state).unwrap().1;
                                let _ = save_user(&user, &conn, &state);
                                break;
                            }
                            break;
                        }
                    }
                });
            }
            Err(error) => match error {
                2 => return Err(StatusCode::FORBIDDEN),
                _ => println!("BAR Error!"),
            },
        }
    }

    let (mut user, conn) = load_user(req.userid, &state).unwrap();
    vec = user
        .vouchers
        .into_iter()
        .filter(|f| f.uuid != req.uuid)
        .collect();
    user.vouchers = vec;

    let _ = state.repo.save(&user, &conn);
    Ok(Json(serde_json::json!({
        "status": "Item Consumed",
    })))
}

fn decrease_flux(user: &mut User, amount: i128) {
    user.flux -= amount;

    if user.dailies[3].last_claimed < Daily::cycle(0) {
        user.todays_flux.1 += amount as u64;
    }
}

#[derive(Deserialize)]
struct CreateAdvanced {
    userid: String,
    id: u64,
    amount: u8,
}
async fn create_advanced(
    State(state): State<AppState>,
    Json(req): Json<CreateAdvanced>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (mut user, conn) = load_user(req.userid.clone(), &state)?;

    let voucher = Voucher::by_id(req.id, &user).map_err(|_| StatusCode::NOT_FOUND)?;
    let cost = (req.amount as u64 * voucher.cost) as i128;
    if user.flux < cost {
        return Err(StatusCode::INSUFFICIENT_STORAGE);
    }

    for idx in 0..req.amount as usize {
        if idx > 25 {
            return Err(StatusCode::FORBIDDEN);
        }
        user.vouchers.push(Voucher::new(
            voucher.id,
            uuid::Uuid::now_v7(),
            voucher.name.clone(),
            voucher.cost,
            voucher.dur,
            true,
            voucher.description.clone(),
            voucher.coeff.clone().into(),
        ));
    }

    decrease_flux(&mut user, cost);

    let _ = state.repo.save(&user, &conn);
    Ok(Json(serde_json::json!({"status": "created"})))
}

#[derive(Deserialize, Serialize, Clone)]
struct Daily {
    id: u8,
    claimable: bool,
    claimed: bool,
    last_claimed: i64,
}
impl Daily {
    fn _init() -> Vec<Self> {
        let mut vec: Vec<Self> = Vec::new();
        for i in 0..5 {
            vec.push(Daily {
                id: i as u8,
                claimable: true,
                claimed: false,
                last_claimed: 0 as i64,
            });
        }
        vec[3].claimable = false;
        vec[4].claimable = false;

        vec
    }
    fn cycle(offset: u8) -> i64 {
        let now = Utc::now();
        let mut cycle_start = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 4 + offset as u32, 0, 0)
            .unwrap();

        if now < cycle_start {
            cycle_start -= Duration::days(1);
        }

        cycle_start.timestamp()
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
    astrum: u16,
    flux: u16,
    astrai: u16,
    vouchers: u16,
}
async fn dailies(
    State(state): State<AppState>,
    Json(req): Json<DailiesReq>,
) -> Result<Json<DailiesResp>, StatusCode> {
    let (mut user, conn) = load_user(req.userid.clone(), &state)?;
    let (mut rw_astrum, mut rw_flux, mut rw_astrai, mut rw_vouchers) = (0, 0, 0, 0);
    let mut ok = false;

    if req.info {
        for daily in user.dailies.iter_mut() {
            if daily.id == 4 {
                if daily.last_claimed >= Daily::cycle(3) {
                    daily.claimable = false;
                    daily.claimed = true;
                } else {
                    daily.claimed = false;
                }
                break;
            }

            if daily.last_claimed >= Daily::cycle(0) {
                daily.claimable = false;
                daily.claimed = true;
            } else {
                daily.claimable = true;
                daily.claimed = false;
            }

            if daily.id == 3 {
                if user.todays_flux.1 >= 324 && !daily.claimed {
                    daily.claimable = true;
                } else {
                    daily.claimable = false;
                }
            }
        }

        ok = true;
    }
    if (req.id < 4 && !req.info) && user.dailies[req.id as usize].last_claimed < Daily::cycle(0) {
        user.dailies[req.id as usize].last_claimed = Utc::now().timestamp();
        user.dailies[req.id as usize].claimed = true;
        user.dailies[req.id as usize].claimable = false;

        if req.id == 3 && user.todays_flux.1 < 324 {
            return Err(StatusCode::FORBIDDEN);
        } else if req.id == 3 {
            user.todays_flux.1 = 0;
        }

        match req.id {
            0 => {
                {
                    let mut bars = state.bars.lock().unwrap();
                    bars.iter_mut().for_each(|bar| {
                        bar.s_reduction = 0.0;
                        bar.s = (bar.s - (60.0 as f64)).max(0.0);
                        bar.tbase = (bar.tbase - (8.0 * 60.0) as f64).max(0.0);
                        bar.is_timing = false;
                        bar.overdrive = false;

                        if bar.id == 5 {
                            bar.reset();
                        }
                    });
                    bars.iter_mut().filter(|f| f.locked).for_each(|bar| {
                        if bar.tbase <= 0.0 {
                            bar.locked = false;
                        }
                    });

                    let bar = bars.get_mut(5).unwrap();
                    *bar = Bar::by_id(5);
                    let mut timer = state.timer.lock().unwrap();
                    *timer = Some(bar.start_idle(state.clone()));
                }

                user.astrum += 240;
                rw_astrum += 240;
            }
            1 => {
                user.astrum += 160;
                user.astrai += 2;

                rw_astrum += 160;
                rw_astrai += 2;
            }
            2 => {
                user.astrum += 80;
                user.flux += 100;

                rw_astrum += 80;
                rw_flux += 100;
            }
            3 => {
                user.astrum += 80;

                rw_astrum += 80;
            }
            _ => {
                println!("This code will never run! Hehe")
            }
        }

        let count = user.dailies.iter().filter(|f| f.claimed).count();
        if count == 3 {
            user.astrum += 100;
            user.astrai += 2;
            user.flux += 100;

            rw_astrum += 100;
            rw_astrai += 2;
            rw_flux += 100;
        }
        ok = true;
    }
    if req.id == 4 && user.dailies[4].claimable {
        user.dailies[req.id as usize].claimable = false;
        user.dailies[req.id as usize].claimed = true;
        user.dailies[req.id as usize].last_claimed = Utc::now().timestamp();

        user.astrum += 1600;
        rw_astrum += 1600;
        rw_vouchers += 1;

        ok = true;
    }

    if req.id < 5 && user.dailies[req.id as usize].claimable {
        if user.dailies.iter().all(|f| f.claimed) {
            user.astrum += 500;
            user.flux += 50;

            rw_astrum += 500;
            rw_flux += 50;
        }
        ok = true;
    }

    if ok {
        let _ = state.repo.save(&user, &conn);
        return Ok(Json(DailiesResp {
            dailies: user.dailies.clone(),
            astrum: rw_astrum,
            flux: rw_flux,
            astrai: rw_astrai,
            vouchers: rw_vouchers,
        }));
    }
    Err(StatusCode::FORBIDDEN)
}

#[derive(Deserialize)]
struct ISRDORequest {
    userid: String,
    description: String,
    coeff: f32,
}
#[derive(Serialize)]
struct ISRDOResponse {
    description: String,
    payout: u16,
    uuid: uuid::Uuid,
}
async fn isrdo(
    State(state): State<AppState>,
    Json(req): Json<ISRDORequest>,
) -> Result<Json<ISRDOResponse>, StatusCode> {
    let (mut user, conn) = load_user(req.userid.clone(), &state)?;
    let payout: u16 = (req.coeff.min(4.96) * 80.0 * 1.24) as u16;

    if user.flux < 80 || user.isrdos.len() == 8 || req.description.is_empty() {
        return Err(StatusCode::FORBIDDEN);
    }

    let isrdo = ISRDO {
        description: req.description.clone(),
        payout: payout,
        uuid: uuid::Uuid::now_v7(),
    };

    user.isrdos.push(isrdo.clone());
    decrease_flux(&mut user, 80_i128);
    let _ = state.repo.save(&user, &conn);

    Ok(Json(ISRDOResponse {
        description: req.description,
        uuid: isrdo.uuid,
        payout: isrdo.payout,
    }))
}

async fn get_isrdos(
    State(state): State<AppState>,
    Json(req): Json<String>,
) -> Result<Json<Vec<ISRDO>>, StatusCode> {
    let (user, _conn) = load_user(req.clone(), &state)?;
    if user.isrdos.len() < 1 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(user.isrdos))
}

#[derive(Deserialize)]
struct ISRDOCompleteReq {
    userid: String,
    uuid: uuid::Uuid,
}
async fn isrdo_complete(
    State(state): State<AppState>,
    Json(req): Json<ISRDOCompleteReq>,
) -> Result<Json<i128>, StatusCode> {
    let (mut user, conn) = load_user(req.userid, &state)?;
    let payout: i128;

    if let Some(isrdo) = user.isrdos.iter_mut().find(|i| i.uuid == req.uuid) {
        payout = isrdo.payout as i128;
        user.isrdos.retain(|f| f.uuid != req.uuid);

        user.flux += payout;
        let _ = state.repo.save(&user, &conn);

        return Ok(Json(payout));
    }

    Err(StatusCode::NOT_FOUND)
}

async fn seven_am_unlock(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    let (mut user, conn) = load_user(get_username(), &state)?;
    let master_key = "X0jItR1QFt38i7kTbexFE2p0pRkrgJmiJdveRmzwl2HazYMQLQRorQg70dVFxcU3WJmpw3qjupACFQTRMJMQh51fcqFTdRIMq5EHS8Ce2e9mq9AAs6B9EA0XuNPQiVrlflCWKy9UGk7StrTh";

    match params.get("key") {
        Some(val) => {
            if val != master_key {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    let daily = &mut user.dailies[4];

    let current_time_utc = Utc::now();
    let current_timestmp = current_time_utc.timestamp();

    let cutoff_start = Utc
        .with_ymd_and_hms(
            current_time_utc.year(),
            current_time_utc.month(),
            current_time_utc.day(),
            7,
            0,
            0,
        )
        .unwrap()
        .timestamp();
    let cutoff = Utc
        .with_ymd_and_hms(
            current_time_utc.year(),
            current_time_utc.month(),
            current_time_utc.day(),
            7,
            15,
            0,
        )
        .unwrap()
        .timestamp();

    if daily.last_claimed < cutoff_start
        && current_timestmp > cutoff_start
        && current_timestmp < cutoff
    {
        daily.claimable = true;
        daily.claimed = false;
    } else {
        return Ok(StatusCode::FORBIDDEN);
    }

    let _ = state
        .repo
        .save(&user, &conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct InfoRequest {
    userid: String,
}
async fn get_user_info(
    State(state): State<AppState>,
    Json(req): Json<InfoRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let (user, _conn) = load_user(req.userid, &state)?;

    Ok(Json(serde_json::json!(
        {
            "astrum": user.astrum,
            "astrai": user.astrai,
            "flux": user.flux,
            "dripstate": user.pause_drip,
        }
    )))
}

async fn remove_new_logo(
    State(state): State<AppState>,
    Json(req): Json<String>,
) -> Result<Json<String>, StatusCode> {
    let (mut user, conn) = load_user(get_username(), &state)?;

    if let Some(voucher) = user.vouchers.iter_mut().find(|f| f.uuid.to_string() == req) {
        voucher.new = false;
    }

    let _ = state
        .repo
        .save(&user, &conn)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json("Flipped".into()))
}

/*
fn drip(b_state: AppState) {
    tokio::spawn(async {
        let state = b_state;
        let mut faucet_failure = false;
        let mut drip_rate: i128 = 0;

        let mut interval = tokio::time::interval(Duration_Time::from_mins(30));
        loop {
            interval.tick().await;
            let (mut user, conn) = load_user(get_username(), &state).unwrap();

            if user.pause_drip {
                continue;
            }

            if Utc::now().hour() >= 3 && Utc::now().hour() < 6 {
                drip_rate = 0;
                faucet_failure = false;
            }

            if faucet_failure {
                drip_rate = drip_rate.max(160);
            } else if drip_rate >= 160 {
                faucet_failure = true;
            }

            if !user.active_timer {
                if user.flux > drip_rate {
                    user.flux -= drip_rate;
                }
                if drip_rate < 140 {
                    drip_rate += 24;
                }
            } else if drip_rate >= 24 {
                drip_rate -= 24
            }

            let _ = state.repo.save(&user, &conn);
        }
    });
}*/

async fn pause_dripper(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    let (mut user, conn) = load_user(
        params
            .get("userid")
            .ok_or(StatusCode::NOT_FOUND)?
            .to_string(),
        &state,
    )?;

    if user.pause_drip {
        user.pause_drip = false;
        save_user(&user, &conn, &state)?;
        Ok(StatusCode::ACCEPTED)
    } else {
        user.pause_drip = true;
        save_user(&user, &conn, &state)?;
        Ok(StatusCode::OK)
    }
}

#[derive(Deserialize, Clone)]
struct BarReq {
    id: u8,
    info: bool,
    userid: String,
}
async fn bars(
    State(state): State<AppState>,
    Json(req): Json<BarReq>,
) -> Result<Json<Vec<Bar>>, StatusCode> {
    let (mut user, conn) = load_user(req.userid.clone(), &state)?;
    if req.info {
        Ok(Json(state.bars.lock().unwrap().clone()))
    } else {
        match req.id {
            255 => Err(StatusCode::FORBIDDEN),
            5 => Err(StatusCode::FORBIDDEN),
            _ => match state.timer.lock() {
                Ok(mut timer_guard) => {
                    if let Some(timer) = timer_guard.take() {
                        timer.abort();

                        let mut result = state.bars.lock().unwrap();

                        let current_id = result.iter().filter(|f| f.is_timing).next().unwrap().id;

                        result.iter_mut().filter(|f| f.is_timing).for_each(|bar| {
                            bar.is_timing = false;
                        });

                        let bars = result.clone();
                        user.bars = bars;

                        let _ = save_user(&user, &conn, &state);
                        if req.id == current_id {
                            *timer_guard = Some(Bar::by_id(5).start_idle(state.clone()));
                            return Err(StatusCode::OK);
                        }
                        if req.id == 5 {
                            return Err(StatusCode::FORBIDDEN);
                        }

                        if !(current_id == req.id) {
                            let status = Bar::by_id(req.id).start_timer(state.clone());

                            if status.is_finished() {
                                return Err(StatusCode::FORBIDDEN);
                            } else {
                                *timer_guard = Some(status);
                                user.active_bar = req.id;
                                let _ = save_user(&user, &conn, &state);
                            }
                            return Err(StatusCode::OK);
                        }

                        Err(StatusCode::ACCEPTED)
                    } else {
                        if req.id == 5 {
                            return Err(StatusCode::FORBIDDEN);
                        }
                        let result = state.bars.lock().unwrap();
                        let status = Bar::by_id(req.id).start_timer(state.clone());
                        let bars = result.clone();
                        user.bars = bars;

                        let _ = save_user(&user, &conn, &state);

                        if status.is_finished() {
                            return Err(StatusCode::FORBIDDEN);
                        } else {
                            *timer_guard = Some(status);
                            user.active_bar = req.id;

                            let _ = save_user(&user, &conn, &state);
                        }
                        Err(StatusCode::OK)
                    }
                }
                Err(_) => Err(StatusCode::FORBIDDEN),
            },
        }
    }
}

fn get_username() -> String {
    "axol999".to_string()
}

#[tokio::main]
async fn main() {
    let repo = SqliteRepo::new("userdata.sql");
    let state = AppState {
        repo: repo.into(),
        timer: Arc::new(Mutex::new(None)),
        bars: Arc::new(Mutex::new(Vec::<Bar>::new())),
    };

    {
        let state_tmp = state.clone();

        let (mut user, conn) = load_user(get_username(), &state_tmp).unwrap();
        //user.vouchers.iter_mut().filter(|f| !f.new).for_each(|f| {
        //    f.new = true;
        //});

        //user.dailies = Daily::init();
        //user.todays_flux.1 = 0;
        //user.templates = Voucher::get_templates();
        user.pause_drip = true;

        let mut bars = state.bars.lock().unwrap();
        *bars = user.bars.clone();

        bars.iter_mut().for_each(|f| {
            f.reset();
        });

        let _ = state_tmp.repo.save(&user, &conn);
    }

    //drip(state.clone());

    let app = Router::new()
        .route("/pull", post(handle_pull))
        .route("/get_user_vouchers", post(get_user_vouchers))
        .route("/purchase", post(purchase))
        .route("/user_funds_info", post(get_user_info))
        .route("/consume", post(consume))
        .route("/create", post(create))
        .route("/dailies", post(dailies))
        .route("/remove_new_logo", post(remove_new_logo))
        .route("/delete_item", post(delete_item))
        .route("/create_advanced", post(create_advanced))
        .route("/isrdo", post(isrdo))
        .route("/get_isrdos", post(get_isrdos))
        .route("/isrdo_complete", post(isrdo_complete))
        .route("/7am_unlock", get(seven_am_unlock))
        .route("/pause_dripper", get(pause_dripper))
        .route("/bars", post(bars))
        .layer(CorsLayer::permissive())
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("11.0.0.2:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
