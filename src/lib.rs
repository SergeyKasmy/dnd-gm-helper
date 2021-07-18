use serde::{Deserialize, Serialize};
use std::io::{self, Write};

type Players = Vec<Player>;

#[derive(Serialize, Deserialize, Default, Debug)]
struct Player {
    name: String,
    class: String,
    stats: Stats,
    skills: Vec<Skill>,
    statuses: Vec<Status>,
    money: i64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Stats {
    strength: i64,
    dexterity: i64,
    poise: i64,
    wisdom: i64,
    intelligence: i64,
    charisma: i64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Skill {
    name: String,
    available_after: u32,
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
    duration: u32,
}

// clear terminal and position the cursor at 0,0
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn get_input() -> String {
    let sin = io::stdin();
    let mut input = String::new();
    if let Err(er) = sin.read_line(&mut input) {
        err(&format!("Couldn't read stdin. {}", er));
    }

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

fn err(text: &str) {
    eprintln!("{}", text);
    get_input();
}

fn edit_player(player: Option<Player>) -> Player {
    fn get_text(old_value: String, stat_name: &str) -> String {
        if !old_value.is_empty() {
            println!("Old {}: {}. Press enter to skip", stat_name, old_value);
        }
        print(&format!("{}: ", stat_name));
        let input = get_input();
        if !old_value.is_empty() && input.is_empty() {
            return old_value;
        }
        input
    }

    fn get_stat_num(old_value: i64, stat_name: &str) -> i64 {
        loop {
            if old_value != 0 {
                println!("Old {}: {}. Press enter to skip", stat_name, old_value);
            }
            print(&format!("{}: ", stat_name));
            let input = get_input();
            if old_value != 0 && input.is_empty() {
                return old_value;
            }
            match input.parse() {
                Ok(num) => return num,
                Err(_) => eprintln!("Not a valid number"),
            }
        }
    }

    //let mut player: Player = Default::default();
    let mut player: Player = player.unwrap_or_default();

    player.name = get_text(player.name, "Name");
    player.class = get_text(player.class, "Class");

    println!("Stats:");
    player.stats.strength = get_stat_num(player.stats.strength, "Strength");
    player.stats.dexterity = get_stat_num(player.stats.dexterity, "Dexterity");
    player.stats.poise = get_stat_num(player.stats.poise, "Poise");
    player.stats.wisdom = get_stat_num(player.stats.wisdom, "Wisdom");
    player.stats.intelligence = get_stat_num(player.stats.intelligence, "Intelligence");
    player.stats.charisma = get_stat_num(player.stats.charisma, "Charisma");

    // edit the existing skills first
    if !player.skills.is_empty() {
        for (i, skill) in player.skills.iter_mut().enumerate() {
            skill.name = get_text(skill.name.clone(), &format!("Skill #{}", i));
            print("Reset CD? ");
            let answer = get_input();
            match answer.as_str() {
                "y" | "yes" => skill.available_after = 0,
                _ => (),
            }
        }
    }

    print("Add new skills? ");
    let answer = get_input();
    match answer.as_str() {
        "y" | "yes" => loop {
            print("Skill name (enter \"q\" to quit): ");
            let input = get_input();
            if input == "q" {
                break;
            }
            player.skills.push(Skill::new(input));
        },
        _ => (),
    }

    player.money = get_stat_num(player.money, "Money");
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
            "....{}. Available after {} moves",
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
            match serde_json::from_str(&json) {
                Ok(data) => players = data,
                Err(er) => {
                    err(&format!("players.json is not a valid json file. {}", er));
                    std::fs::copy(
                        "players.json",
                        format!(
                            "players.json.bak-{}",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        ),
                    )
                    .unwrap();
                }
            };
        }
        Err(er) => err(&format!("Couldn't read from file: {}", er)),
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
            for (i, player) in players.iter().enumerate() {
                let i = i + 1;
                println!("#{}", i);
                print_player(&player);
                println!("-------------------------------------------------");
            }
        } else {
            println!("There are no players.");
        }
        println!("Enter \"e\" to edit, \"d\" to delete, \"a\" to add, and \"q\" to go back");

        if handle_input(|input| {
            match input {
                "a" => {
                    players.push(edit_player(None));
                }
                "d" => {
                    if let Ok(num) = get_input().parse::<i32>() {
                        let num = num - 1;
                        if num < 0 || num as usize >= players.len() {
                            err(&format!("{} is out of bounds", num + 1));
                            return false;
                        }
                        players.remove(num as usize);
                    }
                }
                "e" => {
                    if let Ok(num) = get_input().parse::<i32>() {
                        let num = num - 1;
                        if num < 0 || num as usize >= players.len() {
                            err(&format!("{} is out of bounds", num + 1));
                            return false;
                        }
                        let num = num as usize;
                        let player_to_edit = players.remove(num);
                        players.insert(num, edit_player(Some(player_to_edit)));
                    }
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
