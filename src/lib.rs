mod term;

use serde::{Deserialize, Serialize};
use term::{
    action_enums::{CharacterMenuAction, GameAction, MainMenuAction},
    player_field::PlayerField,
    CharacterMenuMode, Term,
};

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

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum StatType {
    Strength,
    Dexterity,
    Poise,
    Wisdom,
    Intelligence,
    Charisma,
}

// TODO: reimplement using HashMap with StatType as keys
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
    #[allow(dead_code)]
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

fn game_start(term: &mut Term, players: &mut Players) {
    enum NextPlayerState {
        Default,
        Pending,
        Picked(*const Player),
    }
    let mut next_player = NextPlayerState::Default;
    'game: loop {
        if let NextPlayerState::Pending = next_player {
            if let Some(picked_player) = term.pick_player(&players) {
                next_player = NextPlayerState::Picked(picked_player);
            }
        }

        for player in players.iter_mut() {
            if let NextPlayerState::Picked(next_player) = next_player {
                if !std::ptr::eq(next_player, player) {
                    continue;
                }
            }
            loop {
                match term.draw_game(&player) {
                    GameAction::UseSkill => choose_skill_and_use(term, &mut player.skills),
                    GameAction::AddStatus => add_status(term, &mut player.statuses),
                    GameAction::DrainStatusAttacking => {
                        drain_status(player, StatusCooldownType::Attacking)
                    }
                    GameAction::DrainStatusAttacked => {
                        drain_status(player, StatusCooldownType::Attacked)
                    }
                    GameAction::ClearStatuses => player.statuses.clear(),
                    GameAction::ResetSkillsCD => {
                        player
                            .skills
                            .iter_mut()
                            .for_each(|skill| skill.available_after = 0);
                    }
                    GameAction::ManageMoney => manage_money(term, player),
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
    // decrease all statuses duration with the status cooldown type provided
    player.statuses.iter_mut().for_each(|status| {
        if status.status_cooldown_type == status_type && status.duration > 0 {
            status.duration -= 1
        }
    });
    // remove all statuses that have run out = retain all statuses that haven't yet run out
    player.statuses.retain(|status| status.duration > 0);
}

fn choose_skill_and_use(term: &mut Term, skills: &mut Skills) {
    loop {
        let input = match term.choose_skill(&skills) {
            Some(num) => num as usize,
            None => return,
        };
        match skills.get_mut(input) {
            Some(skill) => {
                if skill.available_after == 0 {
                    use_skill(skill);
                } else {
                    term.messagebox("Skill still on cooldown");
                }
                break;
            }
            None => term.messagebox("Number out of bounds"),
        }
    }
}

fn make_move(player: &mut Player) {
    player.skills.iter_mut().for_each(|skill| {
        if skill.available_after > 0 {
            skill.available_after -= 1
        }
    });
    drain_status(player, StatusCooldownType::Normal);
}

fn add_status(term: &Term, statuses: &mut Statuses) {
    if let Some(status) = term.choose_status() {
        statuses.push(status);
    }
}

fn manage_money(term: &Term, player: &mut Player) {
    player.money = player.money + term.get_money_amount();
}

pub fn run() {
    let mut term = Term::new();
    let mut players: Players = vec![];
    let file_contents = std::fs::read_to_string("players.json");
    match file_contents {
        Ok(json) => {
            match serde_json::from_str(&json) {
                Ok(data) => players = data,
                Err(_) => match term.messagebox_yn("The database is corrupted. Continue?") {
                    true => {
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
                    false => return,
                },
            };
        }
        Err(_) => (),
    }

    loop {
        match term.draw_main_menu() {
            MainMenuAction::Play => game_start(&mut term, &mut players),
            MainMenuAction::Edit => character_menu(&mut term, &mut players),
            MainMenuAction::Quit => break,
        }
    }

    std::fs::write("players.json", serde_json::to_string(&players).unwrap()).unwrap();
}

fn character_menu(term: &Term, players: &mut Players) {
    let mut last_selected = None;
    loop {
        match term
            .draw_character_menu(
                CharacterMenuMode::View {
                    selected: last_selected,
                },
                players,
            )
            .unwrap()
        {
            CharacterMenuAction::Add => {
                players.push(Player::default());
                let id = players.len() - 1;
                edit_player(term, players, id);
                last_selected = Some(id);
            }
            CharacterMenuAction::Edit(num) => {
                edit_player(term, players, num);
                last_selected = Some(num);
            }
            CharacterMenuAction::Delete(num) => {
                if term.messagebox_yn("Are you sure?") {
                    players.remove(num);
                    last_selected = num.checked_sub(1);
                }
            }
            CharacterMenuAction::Quit | CharacterMenuAction::Editing { .. } => break,
        }
    }
}

fn edit_player(term: &Term, players: &mut Players, id: usize) {
    let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
    loop {
        //
        // init fields if they don't exist
        match selected_field {
            PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
                let player = players.get_mut(id).unwrap();
                if let None = player.skills.get(skill_id) {
                    player.skills.push(Skill::default())
                }
            }
            _ => (),
        }

        match term.draw_character_menu(
            CharacterMenuMode::Edit {
                selected: id,
                selected_field,
            },
            players,
        ) {
            Some(CharacterMenuAction::Editing {
                buffer,
                field_offset,
            }) => {
                let player = players.get_mut(id).unwrap();
                match selected_field {
                    PlayerField::Name => {
                        let _ = std::mem::replace(&mut player.name, buffer);

                        // TODO: modify inplace
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                    PlayerField::Stat(stat) => {
                        let current_stat = match stat {
                            StatType::Strength => &mut player.stats.strength,
                            StatType::Dexterity => &mut player.stats.dexterity,
                            StatType::Poise => &mut player.stats.poise,
                            StatType::Wisdom => &mut player.stats.wisdom,
                            StatType::Intelligence => &mut player.stats.intelligence,
                            StatType::Charisma => &mut player.stats.charisma,
                        };

                        match buffer.parse::<i64>() {
                            Ok(result) => {
                                *current_stat = result;
                                selected_field = match field_offset.unwrap_or(1) {
                                    1 => selected_field.next(),
                                    -1 => selected_field.prev(),
                                    _ => selected_field,
                                }
                            }
                            Err(_) => (),
                        }
                    }
                    PlayerField::SkillName(skill_id) => {
                        // TODO: pop a skill if the buffer is empty
                        let _ = std::mem::replace(&mut player.skills[skill_id].name, buffer);
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                    PlayerField::SkillCD(skill_id) => {
                        match buffer.parse::<u32>() {
                            Ok(num) => {
                                player.skills[skill_id].cooldown = num;
                            }
                            Err(_) => (),
                        }
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                }
            }
            None => return,
            _ => unreachable!(),
        }
    }
}
