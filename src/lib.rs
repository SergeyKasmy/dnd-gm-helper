use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Players(Vec<Player>);

#[derive(Serialize, Deserialize, Debug)]
struct Player {
    name: String,
    class: String,
    stats: Stats,
    skills: Vec<Skill>,
    statuses: Vec<Status>,
    money: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    strength: u8,
    dexterity: u8,
    poise: u8,
    wisdom: u8,
    intelegence: u8,
    charisma: u8,
}

#[derive(Serialize, Deserialize, Debug)]
struct Skill {
    name: String,
    available_after: u8,
}

#[derive(Serialize, Deserialize, Debug)]
enum StatusType {
    Luck,
    Stun,
}

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    status_type: StatusType,
    duration: u8,
}

// clear terminal and position the cursor at 0,0
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

// returns true if the user asked to quit
fn handle_input<F: Fn(&str) -> bool>(handler: F) -> bool {
    let sin = std::io::stdin();
    let mut input = String::new();
    sin.read_line(&mut input).expect("Couldn't read stdin");

    let input = input.trim();
    handler(input)
}

fn print_player(player: &Player) {
    println!("Name: {}", player.name);
    println!("Class: {}", player.class);

    println!("Skills:");
    for skill in &player.skills {
        println!(
            "....Skill: {}, Available after: {} moves",
            skill.name, skill.available_after
        );
    }

    println!("Statuses:");
    for status in &player.statuses {
        println!(
            "....Status: {:?}, Still active for {} moves",
            status.status_type, status.duration
        );
    }

    println!("Money: {}", player.money);
}

pub fn run() {
    let mut players = Players { 0: vec![] };
    let file_contents = std::fs::read_to_string("players.json");
    match file_contents {
        Ok(json) => {
            players = serde_json::from_str(&json).expect("Not a valid json file players.json")
        }
        Err(err) => {
            eprintln!("Couldn't read from file: {}", err);
            players.0.push(Player {
                name: String::from("Test Name"),
                class: String::from("Test Class"),
                stats: Stats {
                    strength: 6,
                    dexterity: 6,
                    poise: 6,
                    wisdom: 1,
                    intelegence: 6,
                    charisma: 6,
                },
                skills: vec![
                    Skill {
                        name: String::from("Test Skill 1"),
                        available_after: 0,
                    },
                    Skill {
                        name: String::from("Test Skill 2"),
                        available_after: 0,
                    },
                ],
                statuses: vec![Status {
                    status_type: StatusType::Luck,
                    duration: 2,
                }],
                money: 50,
            })
        }
    }

    loop {
        clear_screen();
        println!("1. Start game");
        println!("2. View/Edit characters");
        println!("q. Quit");

        if handle_input(|input| {
            match input {
                "2" => character_menu(Some(&players)),
                "q" => return true,
                _ => (),
            }

            false
        }) {
            break;
        }
    }

    std::fs::write("players.json", serde_json::to_string(&players).unwrap()).unwrap();
}

fn character_menu(players: Option<&Players>) {
    loop {
        clear_screen();
        if let Some(players) = players {
            for player in &players.0 {
                print_player(&player);
            }
        } else {
            println!("There are no players.");
        }

        if handle_input(|input| {
            match input {
                "q" => return true,
                _ => (),
            }

            false
        }) {
            break;
        }
    }
}
