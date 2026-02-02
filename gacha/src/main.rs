use gacha_protocol::machine::{get_string, Rarities};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
};

#[derive(Parser, Debug)]
struct Args {
    action: String,
    amount: usize,
    effort: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
    tickets: u64,
    sss_pity: i16,
    ss_pity: i16,
    s_pity: i16,
    inc_sss: f64,
    inc_ss: f64,
    inc_s: f64,
    has_slip: bool,

    total_pulls: u128,
}
impl Player {
    fn get_player_ref(&self) -> &Player {
        return self;
    }
    fn new(name: String) -> Self {
        Self {
            name,
            tickets: 0,
            sss_pity: 0,
            ss_pity: 0,
            s_pity: 0,
            inc_sss: 0.0,
            inc_ss: 0.0,
            inc_s: 0.0,
            has_slip: false,
            total_pulls: 0,
        }
    }
}

struct JsonMembers {
    members: Vec<Player>,
}
fn save_user(player: &Player) -> std::io::Result<()> {
    let file = File::create("/home/jwm/.programs/gacha/user_data.json").unwrap();
    let mut writer = BufWriter::new(file);

    serde_json::to_writer_pretty(&mut writer, player)?;
    Ok(())
}

fn main() {
    let data = fs::read_to_string("/home/jwm/.programs/gacha/user_data.json");
    let mut player = Player::new("axol".to_string());
    let mut wishes: Option<Vec<Rarities>> = None;

    match data {
        Ok(data) => {
            if !data.is_empty() {
                player = serde_json::from_str(&data).unwrap();
            }
        }
        Err(_) => {
            let _ = File::create("./user_data.json").unwrap();
        }
    }

    let args = Args::parse();
    match args.action.as_str() {
        "add" => {
            player.tickets = player.tickets + args.amount as u64;
            println!("Added {} tickets to {}'s wallet!", args.amount, player.name);
            let _ = save_user(&player);
            return;
        }
        "pull" => {
            wishes = match args.amount {
                1 => Some(machine::handle_pull(&mut player)),
                10 => Some(machine::handle_pull_ten(&mut player)),
                //10000 => Some(machine::handle_pull_h(&mut player)),
                _ => {
                    println!("Invalid pull count");
                    panic!();
                }
            }
        }
        _ => {
            println!("Unknown command!");
            panic!()
        }
    }
    let _ = save_user(&player);

    match wishes {
        None => {
            println!("Internal Err, no wishes made.");
            panic!();
        }
        _ => {
            println!("Successfully Performed Pulls");
        }
    }
    let mut wishes = wishes.unwrap();
    match wishes[0] {
        Rarities::NoTickets => {
            println!("Not Enough Tickets Available!");
            panic!();
        }
        _ => print!(""),
    };

    for x in wishes.iter_mut() {
        let won = get_string(&x);
        match won {
            "A" => println!("You pulled an {}", won),
            "SS" => println!("You pulled a {}, Wow!", won),
            "Mythic" => println!("You pulled a {}, Unreal!", won),
            _ => println!("You pulled a {}", won),
        }
    }

    /*
        let mut mythic = 0;
        let mut ss = 0;
        let mut s = 0;
        let mut a = 0;
        let mut b = 0;
        let mut c = 0;

        for x in wishes.iter() {
            match machine::get_string(&x) {
                "Mythic" => mythic += 1,
                "SS" => ss += 1,
                "S" => s += 1,
                "A" => a += 1,
                "B" => b += 1,
                _ => c += 1,
            }
        }
        println!("Mythics: {}", mythic);
        println!("SS: {}", ss);
        println!("S: {}", s);
        println!("A: {}", a);
        println!("B: {}", b);
        println!("C: {}", c);
    */
