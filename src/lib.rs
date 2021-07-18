use std::io::{self, Write};
use serde::{Deserialize, Serialize};

type Players = Vec<Player>;

#[derive(Serialize, Deserialize, Default, Debug)]
struct Player {
    name: String,
    class: String,
    stats: Stats,
    skills: Vec<Skill>,
    statuses: Vec<Status>,
    money: u32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Stats {
    strength: u8,
    dexterity: u8,
    poise: u8,
    wisdom: u8,
    intelligence: u8,
    charisma: u8,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Skill {
    name: String,
    available_after: u8,
}

impl Skill {
    fn new(name: String) -> Skill {
        Skill {
            name,
            available_after: 0,
        }
    }
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

fn get_input() -> String {
    let sin = io::stdin();
    let mut input = String::new();
    sin.read_line(&mut input).expect("Couldn't read stdin");

    input.trim().to_string()
}

// returns true if the user asked to quit
fn handle_input<F: FnMut(&str) -> bool>(mut handler: F) -> bool {
    handler(&get_input())
}

fn print(text: &str) {
    let mut sout = io::stdout();
    print!("{}", text);
    sout.flush().unwrap();
}

fn add_player() -> Player {
    let mut player: Player = Default::default();
    println!("{:?}", player);

    print("Name: ");
    player.name = get_input();

    print("Class: ");
    player.class = get_input();

    println!("Stats:");
    print("....Strength: ");
    player.stats.strength = get_input().parse().unwrap();
    print("....Dexterity: ");
    player.stats.dexterity = get_input().parse().unwrap();
    print("....Poise: ");
    player.stats.poise = get_input().parse().unwrap();
    print("....Wisdom: ");
    player.stats.wisdom = get_input().parse().unwrap();
    print("....Intelligence: ");
    player.stats.intelligence = get_input().parse().unwrap();
    print("....Charisma: ");
    player.stats.charisma = get_input().parse().unwrap();

    loop {
        print("Skill name(enter \"q\" to skip): ");
        let input = get_input();
        if input == "q" { break; }
        player.skills.push(Skill::new(input));
    }

    print("Money: ");
    player.money = get_input().parse().unwrap();

    player
}

fn print_player(player: &Player) {
    println!("Name: {}", player.name);
    println!("Class: {}", player.class);

    println!("Stats:");
    println!("....Strength: {}", player.stats.strength);
    println!("....Dexterity: {}", player.stats.dexterity);
    println!("....Poise: {}", player.stats.poise);
    println!("....Wisdom: {}", player.stats.wisdom);
    println!("....Intelligence: {}", player.stats.intelligence);
    println!("....Charisma: {}", player.stats.charisma);

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
    let mut players: Players = vec![];
    let file_contents = std::fs::read_to_string("players.json");
    match file_contents {
        Ok(json) => {
            players = serde_json::from_str(&json).expect("Not a valid json file players.json")
        }
        Err(err) => {
            eprintln!("Couldn't read from file: {}", err);
            players.push(Player {
                name: String::from("Test Name"),
                class: String::from("Test Class"),
                stats: Stats {
                    strength: 6,
                    dexterity: 6,
                    poise: 6,
                    wisdom: 1,
                    intelligence: 6,
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
                "2" => character_menu(&mut players),
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

fn character_menu(players: &mut Players) {
    loop {
        clear_screen();
        if !players.is_empty() {
            for player in players.iter() {
                print_player(&player);
            }
        } else {
            println!("There are no players.");
        }
        println!("\na. Add a new player");

        if handle_input(|input| {
            match input {
                "a" => {
                    players.push(add_player());
                }
                "q" => return true,
                _ => (),
            }

            false
        }) {
            break;
        }
    }
}
