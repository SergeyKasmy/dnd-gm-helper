mod term;

use term::{Tui, MainMenuAction, CharacterMenuAction};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

type Players = Vec<Player>;
type Skills = Vec<Skill>;
type Statuses = Vec<Status>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Player {
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
    cooldown: u32,
    available_after: u32,
}

impl Skill {
    fn new(name: String, cooldown: u32) -> Skill {
        Skill {
            name,
            cooldown,
            available_after: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum StatusType {
    Discharge,
    FireAttack,
    FireShield,
    IceShield,
    Blizzard,
    Fusion,
    Luck,

    Knockdown,
    Poison,
    Stun,
}

#[derive(Serialize, Deserialize, Debug)]
struct Status {
    status_type: StatusType,
    status_cooldown_type: StatusCooldownType,
    duration: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
enum StatusCooldownType {
    Normal,
    Attacking,
    Attacked,
}

fn get_input() -> String {
    unimplemented!();
}

fn clear_screen() {
    unimplemented!();
}

fn print(_text: &str) {
    unimplemented!();
}

fn game_start(players: &mut Players) {
    loop {
        for player in players.iter_mut() {
            loop {
                clear_screen();
                print_player(&player, false);
                println!("Use skill: \"s\", Add status: \"a\", Drain status after attacking: \"b\", after getting attacked: \"n\", Manage money: \"m\", Next move: \" \", Skip move: \"p\", Quit game: \"q\"");
                match get_input().as_str() {
                    "s\n" | "s\r\n" => choose_skill_and_use(&mut player.skills),
                    "a\n" | "a\r\n" => add_status(&mut player.statuses),
                    "b\n" | "b\r\n" => drain_status(player, StatusCooldownType::Attacking),
                    "n\n" | "n\r\n" => drain_status(player, StatusCooldownType::Attacked),
                    "m\n" | "m\r\n" => manage_money(player),
                    "c\n" | "c\r\n" => player.statuses.clear(),
                    " \n" | " \r\n" => {
                        make_move(player);
                        break;
                    }
                    "p\n" | "p\r\n" => break,
                    "q\n" | "q\r\n" => return,
                    _ => (),
                }
            }
        }
    }
}

fn use_skill(skill: &mut Skill) {
    skill.available_after = skill.cooldown;
}

fn drain_status(player: &mut Player, status_type: StatusCooldownType) {
    for status in player.statuses.iter_mut() {
        if status.status_cooldown_type == status_type {
            if status.duration > 0 {
                status.duration = status.duration - 1;
            }
        }
    }

    let mut i = 0;
    while i < player.statuses.len() {
        if player.statuses[i].duration <= 0 {
            player.statuses.remove(i);
        } else {
            i += 1;
        }
    }
}

fn choose_skill_and_use(skills: &mut Skills) {
    for (i, skill) in skills.iter_mut().enumerate() {
        println!("#{}: {}", i + 1, skill.name);
    }

    loop {
        let input = get_input();
        let input: usize = loop {
            match input.trim().parse() {
                Ok(num) => break num,
                Err(_) => Tui::err("Not a valid number"),
            }
        };

        // TODO: Handle unwrap
        match skills.get_mut(input - 1) {
            Some(skill) => {
                if skill.available_after == 0 {
                    use_skill(skill);
                } else {
                    Tui::err("Skill still on cooldown");
                }
                break;
            }
            None => Tui::err("Number out of bounds"),
        }
    }
}

fn make_move(player: &mut Player) {
    for skill in &mut player.skills {
        if skill.available_after > 0 {
            skill.available_after = skill.available_after - 1;
        }
    }

    for status in &mut player.statuses {
        if status.status_cooldown_type == StatusCooldownType::Normal && status.duration > 0 {
            status.duration = status.duration - 1;
        }
    }

    let mut i = 0;
    while i < player.statuses.len() {
        if player.statuses[i].duration <= 0 {
            player.statuses.remove(i);
        } else {
            i += 1;
        }
    }
}

fn add_status(statuses: &mut Statuses) {
    println!("Choose a status:");
    println!("Buffs:");
    println!("#1 Discharge");
    println!("#2 Fire Attack");
    println!("#3 Fire Shield");
    println!("#4 Ice Shield");
    println!("#5 Blizzard");
    println!("#6 Fusion");
    println!("#7 Luck");
    println!("Debuffs:");
    println!("#8 Knockdown");
    println!("#9 Poison");
    println!("#0 Stun");

    let status_type = loop {
        match get_input().trim() {
            "1" => break StatusType::Discharge,
            "2" => break StatusType::FireAttack,
            "3" => break StatusType::FireShield,
            "4" => break StatusType::IceShield,
            "5" => break StatusType::Blizzard,
            "6" => break StatusType::Fusion,
            "7" => break StatusType::Luck,
            "8" => break StatusType::Knockdown,
            "9" => break StatusType::Poison,
            "0" => break StatusType::Stun,
            "q" => return,
            _ => continue,
        };
    };

    print("Status cooldown type (1 for normal, 2 for on getting attacked, 3 for attacking): ");
    let status_cooldown_type = loop {
        match get_input().trim().parse::<u8>() {
            Ok(num) => break num,
            Err(_) => Tui::err("Not a valid number"),
        }
    };

    let status_cooldown_type = match status_cooldown_type {
        1 => StatusCooldownType::Normal,
        2 => StatusCooldownType::Attacked,
        3 => StatusCooldownType::Attacking,
        _ => {
            Tui::err("Not a valid cooldown type");
            return;
        }
    };

    print("Enter status duration: ");
    let duration = loop {
        match get_input().trim().parse::<u32>() {
            Ok(num) => break num,
            Err(_) => eprintln!("Number out of bounds"),
        }
    };

    statuses.push(Status {
        status_type,
        status_cooldown_type,
        duration,
    })
}

fn manage_money(player: &mut Player) {
    print("Add or remove money (use + or - before the amount): ");
    let input = get_input().trim().to_string();

    if input.len() < 2 {
        Tui::err(&format!(
            "{} is not a valid input. Good examples: +500, -69",
            input
        ));
        return;
    }

    let mut op = '.';
    let mut amount = String::new();

    for (i, ch) in input.chars().enumerate() {
        if i == 0 {
            op = ch;
        } else {
            amount.push(ch);
        }
    }

    let amount: i64 = match amount.parse() {
        Ok(num) => num,
        Err(_) => {
            Tui::err("Not a valid number");
            return;
        }
    };

    player.money = match op {
        '+' => player.money + amount,
        '-' => player.money - amount,
        _ => {
            Tui::err(&format!("{} is not a valid operator (+ or -)", op));
            return;
        }
    }
}


fn print_player(player: &Player, verbose: bool) {
    println!("Name: {}", player.name);
    if verbose {
        println!("Class: {}", player.class);
    }

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
            "....{}. CD: {}. Available after {} moves",
            skill.name, skill.cooldown, skill.available_after
        );
    }

    println!("Statuses:");
    for status in &player.statuses {
        println!(
            "....{:?}, Still active for {} moves",
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
                    Tui::err(&format!("players.json is not a valid json file. {}", er));
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
        Err(er) => Tui::err(&format!("Couldn't read from file: {}", er)),
    }

    let mut tui = Tui::new();
    loop {
        match tui.draw_main_menu() {
            MainMenuAction::Play => game_start(&mut players),
            MainMenuAction::Edit => character_menu(&mut tui, &mut players),
            MainMenuAction::Quit => break,
        }
    }

    std::fs::write("players.json", serde_json::to_string(&players).unwrap()).unwrap();
}

fn character_menu(term: &mut Tui, players: &mut Players) {
    loop {
        match term.draw_character_menu(&players) {
            CharacterMenuAction::Add => {
                players.push(term.edit_player(None));
            }
            CharacterMenuAction::Edit(num) => {
                    let num = num - 1;
                    if num < 0 || num as usize >= players.len() {
                        Tui::err(&format!("{} is out of bounds", num + 1));
                        break;
                    }
                    let num = num as usize;
                    let player_to_edit = players.remove(num);
                    players.insert(num, term.edit_player(Some(player_to_edit)));
            }
            CharacterMenuAction::Delete(num) => {
                    let num = num - 1;
                    if num < 0 || num as usize >= players.len() {
                        Tui::err(&format!("{} is out of bounds", num + 1));
                        break;
                    }
                    players.remove(num as usize);
            }
            CharacterMenuAction::Quit => break,
        }
    }
}
