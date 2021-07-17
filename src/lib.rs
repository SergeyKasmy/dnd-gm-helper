struct Players(Vec<Player>);

#[derive(Debug)]
struct Player {
    name: String,
    class: String,
    skills: Vec<Skill>,
    statuses: Vec<Status>,
    money: u32
}

#[derive(Debug)]
struct Skill {
    name: String,
    available_after: u8,
}

#[derive(Debug)]
enum StatusType {
    Luck,
    Stun,
}

#[derive(Debug)]
struct Status {
    status_type: StatusType,
    duration: u8,
}

// clear terminal and position the cursor at 0,0
fn clear_screen() {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn print_player(player: &Player) {
    println!("Name: {}", player.name);
    println!("Class: {}", player.class);

    println!("Skills:");
    for skill in &player.skills {
        println!("....Skill: {}, Available after: {} moves", skill.name, skill.available_after);
    }

    println!("Statuses:");
    for status in &player.statuses {
        println!("....Status: {:?}, Still active for {} moves", status.status_type, status.duration);
    }

    println!("Money: {}", player.money);
}

pub fn run() {
    let player = Player { name: String::from("Ciren"), class: String::from("Полу-человек полу-эльф"), skills: vec![ Skill { name: String::from("Test Skill 1"), available_after: 0 }, Skill { name: String::from("Test Skill 2"), available_after: 0 } ], statuses: vec![Status { status_type: StatusType::Luck, duration: 2 }], money: 50 };
    let players = Players(vec![player]);
    loop {
        clear_screen();
        println!("1. Start game");
        println!("2. View/Edit characters");
        println!("q. Quit");

        let sin = std::io::stdin();
        let mut input = String::new();
        sin.read_line(&mut input).unwrap();

        let input = input.trim();
        match input {
            "2" => character_menu(Some(&players)),
            "q" => break,
            _ => ()
        }
    }
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

        let sin = std::io::stdin();
        let mut input = String::new();
        sin.read_line(&mut input).unwrap();

        let input = input.trim();
        match input {
            "q" => return,
            _ => (),
        }
    }
}
