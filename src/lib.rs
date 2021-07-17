#[derive(Debug)]
struct Player {
    name: String,
    class: String,
    skills: Vec<Skill>,
    status: Vec<Status>,
    money: u32
}

#[derive(Debug)]
struct Skill {
    name: String,
    moves_unavailable: u8,
}

#[derive(Debug)]
enum StatusType {
    Luck,
    Stun,
}

#[derive(Debug)]
struct Status {
    status: StatusType,
    duration: u8,
}

pub fn run() {
    let test = Player { name: String::from("Ciren"), class: String::from("Полу-человек полу-эльф"), skills: vec![ Skill { name: String::from("Test Skill 1"), moves_unavailable: 0 }, Skill { name: String::from("Test Skill 2"), moves_unavailable: 0 } ], status: vec![Status { status: StatusType::Luck, duration: 2 }], money: 50 };

    println!("{:?}", test);
}
