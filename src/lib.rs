mod term;

use serde::{Deserialize, Serialize};
use term::{CharacterMenuAction, CharacterMenuMode, GameAction, MainMenuAction, Tui};

type Players = Vec<Player>;
type Skills = Vec<Skill>;
type Statuses = Vec<Status>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Player {
    name: String,
    stats: Stats,
    skills: Vec<Skill>,
    statuses: Vec<Status>,
    money: i64,
}

#[derive(Copy, Clone, Debug)]
enum StatType {
    Strength,
    Dexterity,
    Poise,
    Wisdom,
    Intelligence,
    Charisma,
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
pub struct Skill {
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
pub struct Status {
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

fn game_start(tui: &mut Tui, players: &mut Players) {
    enum NextPlayerState {
        Default,
        Pending,
        Picked(*const Player),
    }
    let mut next_player = NextPlayerState::Default;
    'game: loop {
        if let NextPlayerState::Pending = next_player {
            next_player = NextPlayerState::Picked(Tui::pick_player(&players));
        }

        for player in players.iter_mut() {
            if let NextPlayerState::Picked(next_player) = next_player {
                if !std::ptr::eq(next_player, player) {
                    continue;
                }
            }
            loop {
                match tui.draw_game(&player) {
                    GameAction::UseSkill => choose_skill_and_use(&mut player.skills),
                    GameAction::AddStatus => add_status(&mut player.statuses),
                    GameAction::DrainStatusAttacking => {
                        drain_status(player, StatusCooldownType::Attacking)
                    }
                    GameAction::DrainStatusAttacked => {
                        drain_status(player, StatusCooldownType::Attacked)
                    }
                    GameAction::ClearStatuses => player.statuses.clear(),
                    GameAction::ResetSkillsCD => {
                        for skill in player.skills.iter_mut() {
                            skill.available_after = 0;
                        }
                    }
                    GameAction::ManageMoney => manage_money(player),
                    GameAction::MakeTurn => {
                        make_move(player);
                        break;
                    }
                    GameAction::SkipTurn => break,
                    GameAction::NextPlayerPick => {
                        next_player = NextPlayerState::Pending;
                        continue 'game;
                    }
                    GameAction::Quit => return,
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
    loop {
        let input = match Tui::choose_skill(&skills) {
            Some(num) => num as usize,
            None => return,
        };
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
    if let Some(status) = Tui::choose_status() {
        statuses.push(status);
    }
}

fn manage_money(player: &mut Player) {
    player.money = player.money + Tui::get_money_amount();
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
            MainMenuAction::Play => game_start(&mut tui, &mut players),
            MainMenuAction::Edit => character_menu(&mut tui, &mut players),
            MainMenuAction::Quit => break,
        }
    }

    std::fs::write("players.json", serde_json::to_string(&players).unwrap()).unwrap();
}

fn character_menu(term: &mut Tui, players: &mut Players) {
    loop {
        match term
            .draw_character_menu(CharacterMenuMode::View, players)
            .unwrap()
        {
            CharacterMenuAction::Add => {
                term.draw_character_menu(CharacterMenuMode::Add, players);
            }
            CharacterMenuAction::Edit(num) => {
                let player_to_edit = players.remove(num);
                players.insert(num, term.edit_player(Some(player_to_edit)));
            }
            CharacterMenuAction::Delete(num) => {
                players.remove(num);
            }
            CharacterMenuAction::Quit => break,
        }
    }
}
