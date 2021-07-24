mod term;

use term::{Tui, MainMenuButton};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};

type Players = Vec<Player>;
type Skills = Vec<Skill>;
type Statuses = Vec<Status>;

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

    input
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
                    " \n" | " \r\n" => 
                    {
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
                Err(_) => err("Not a valid number"),
            }
        };

        // TODO: Handle unwrap
        match skills.get_mut(input - 1) {
            Some(skill) => {
                if skill.available_after == 0 {
                    use_skill(skill);
                } else {
                    err("Skill still on cooldown");
                }
                break
            }
            None => err("Number out of bounds"),
        }
    }
}

fn make_move(player: &mut Player) {
    for skill in &mut player.skills {
        if skill.available_after > 0 { skill.available_after = skill.available_after - 1; }
    }

    for status in &mut player.statuses {
        if status.status_cooldown_type == StatusCooldownType::Normal && status.duration > 0 { status.duration = status.duration - 1; }
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
            Err(_) => err("Not a valid number"),
        }
    };

    let status_cooldown_type = match status_cooldown_type {
        1 => StatusCooldownType::Normal,
        2 => StatusCooldownType::Attacked,
        3 => StatusCooldownType::Attacking,
        _ => {
            err("Not a valid cooldown type");
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

    statuses.push(Status { status_type, status_cooldown_type, duration })
}

fn manage_money(player: &mut Player) {
    print("Add or remove money (use + or - before the amount): ");
    let input = get_input().trim().to_string();

    if input.len() < 2 {
        err(&format!("{} is not a valid input. Good examples: +500, -69", input));
        return;
    }

    let mut op = '.';
    let mut amount = String::new();

    for (i, ch) in input.chars().enumerate() {
        if i == 0 { op = ch; } else {
            amount.push(ch);
        }
    }

    let amount: i64 = match amount.parse() {
        Ok(num) => num,
        Err(_) => {
            err("Not a valid number");
            return
        }
    };

    player.money = match op {
        '+' => player.money + amount,
        '-' => player.money - amount,
        _ => {
            err(&format!("{} is not a valid operator (+ or -)", op));
            return;
        },
    } 
}

fn edit_player(player: Option<Player>) -> Player {
    fn get_text(old_value: String, stat_name: &str) -> String {
        if !old_value.is_empty() {
            println!("Old {}: {}. Press enter to skip", stat_name, old_value);
        }
        print(&format!("{}: ", stat_name));
        let input = get_input().trim().to_string();
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
            let input = get_input().trim().to_string();
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
            // TODO: parse i64 to u32 correctly
            skill.cooldown = get_stat_num(skill.cooldown as i64, "Cooldown") as u32;
            print("Reset existing cooldown to 0? ");
            let answer = get_input();
            match answer.trim() {
                "y" | "yes" => skill.available_after = 0,
                _ => (),
            }
        }
    }

    print("Add new skills? ");
    let answer = get_input();
    match answer.trim() {
        "y" | "yes" => loop {
            print("Skill name (enter \"q\" to quit): ");
            let name = get_input().trim().to_string();
            if name == "q" {
                break;
            }
            print("Skill cooldown: ");
            let cd = loop {
                match get_input().trim().parse::<u32>() {
                    Ok(num) => break num,
                    Err(_) => err("Not a valid number"),
                };
            };
            player.skills.push(Skill::new(name, cd));
        },
        _ => (),
    }

    player.money = get_stat_num(player.money, "Money");
    player
}

fn print_player(player: &Player, verbose: bool) {
    println!("Name: {}", player.name);
    if verbose { println!("Class: {}", player.class); }

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
        match Tui::new().draw_main_menu() {
            MainMenuButton::Play => game_start(&mut players),
            MainMenuButton::Edit => character_menu(&mut players),
            MainMenuButton::Quit => break,
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
                print_player(&player, true);
                println!("-------------------------------------------------");
            }
        } else {
            println!("There are no players.");
        }
        println!("Edit: \"e\", Delete: \"d\", Add: \"a\", Go back: \"q\"");

        if handle_input(|input| {
            match input.trim() {
                "a" => {
                    players.push(edit_player(None));
                }
                "d" => {
                    if let Ok(num) = get_input().trim().parse::<i32>() {
                        let num = num - 1;
                        if num < 0 || num as usize >= players.len() {
                            err(&format!("{} is out of bounds", num + 1));
                            return false;
                        }
                        players.remove(num as usize);
                    }
                }
                "e" => {
                    if let Ok(num) = get_input().trim().parse::<i32>() {
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
