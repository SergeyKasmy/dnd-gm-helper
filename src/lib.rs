// TODO: add game chooser w/ separate stats and players
// TODO: add force crash with vars and bt
pub mod action_enums;
mod player;
pub mod player_field;
mod skill;
mod stats;
mod status;
mod term;

use action_enums::{CharacterMenuAction, GameAction, MainMenuAction};
use once_cell::sync::Lazy;
use player::{Player, Players};
use player_field::PlayerField;
use skill::Skill;
use stats::StatList;
use status::StatusCooldownType;
use std::collections::HashMap;
use std::sync::Mutex;
use term::{CharacterMenuMode, Term};

// TODO: that's... not so good. Don't do such stupid things next time, mate
pub static STAT_LIST: Lazy<Mutex<StatList>> = Lazy::new(|| {
    let mut stats: HashMap<usize, String> = HashMap::new();
    stats.insert(0, "Strength".to_string());
    stats.insert(1, "Dexterity".to_string());
    stats.insert(2, "Poise".to_string());
    stats.insert(3, "Wisdom".to_string());
    stats.insert(4, "Intelligence".to_string());
    stats.insert(5, "Charisma".to_string());
    Mutex::new(StatList::new(stats))
});

macro_rules! get_player {
    ($players:ident, $i:expr) => {
        $players
            .get($i)
            .ok_or(())
            .map_err(|_| log::error!("{} is not a valid id", $i))
            .unwrap()
    };
}

macro_rules! get_player_mut {
    ($players:ident, $i:expr) => {
        $players
            .get_mut($i)
            .ok_or(())
            .map_err(|_| log::error!("{} is not a valid id", $i))
            .unwrap()
    };
}

pub fn run() {
    log::debug!("Starting...");
    let term = Term::new();
    let mut players = Players::default();
    let file_contents = std::fs::read_to_string("players.json");
    if let Ok(json) = file_contents.map_err(|e| log::info!("players.json could not be read: {}", e))
    {
        match serde_json::from_str(&json) {
            Ok(data) => {
                log::debug!("Read from the db: {:#?}", data);
                players = data;
            }
            Err(_) => {
                // TODO: convert old format with Vec to the new with HashMap
                log::error!("The database is corrupted");
                if term.messagebox_yn("The database is corrupted. Continue?") {
                    let db_bak = format!(
                        "players.json.bak-{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    );
                    log::info!("Coping the old corrupted db to {}", db_bak);
                    let _ = std::fs::copy("players.json", db_bak)
                        .map_err(|e| log::error!("Error copying: {}", e));
                } else {
                    return;
                }
            }
        }
    }

    loop {
        match term.draw_main_menu() {
            MainMenuAction::Play => game_start(&term, &mut players),
            MainMenuAction::Edit => character_menu(&term, &mut players),
            MainMenuAction::Quit => break,
        }
    }

    log::debug!("Saving player data to the db");
    let _ = std::fs::write("players.json", serde_json::to_string(&players).unwrap())
        .map_err(|e| log::error!("Error saving to the db: {}", e));
    log::debug!("Exiting...");
}

fn game_start(term: &Term, players: &mut Players) {
    log::debug!("In the game menu...");
    enum NextPlayerState {
        Default,
        Pending,
        Picked(*const Player),
    }
    let mut next_player = NextPlayerState::Default;

    // TODO: do this only if player_order is empty
    let player_order = players
        .as_vec()
        .iter()
        .map(|(id, _)| *id)
        .collect::<Vec<usize>>();
    'game: loop {
        if let NextPlayerState::Pending = next_player {
            log::debug!("Pending a next player change.");
            if let Some(picked_player) = term.pick_player(players) {
                log::debug!("Picked next player: {}", picked_player.name);
                next_player = NextPlayerState::Picked(picked_player);
            }
        }

        //for (id, player) in players.iter_mut() {
        for &id in player_order.iter() {
            if let NextPlayerState::Picked(next_player) = next_player {
                let player = get_player!(players, id);
                if !std::ptr::eq(next_player, player) {
                    log::debug!("Skipping player {}", player.name);
                    continue;
                }
            }
            log::debug!("Current turn: {} #{}", get_player!(players, id).name, id);
            loop {
                match term.draw_game(get_player!(players, id)) {
                    // TODO: combine lesser used options into a menu
                    // TODO: use skills on others -> adds status
                    // TODO: rename "Drain status" to "Got hit"/"Hit mob"
                    GameAction::UseSkill => {
                        // TODO: rework
                        let skills = &mut get_player_mut!(players, id).skills;
                        log::debug!("Choosing a skill to use");
                        loop {
                            let input = match term.choose_skill(skills) {
                                Some(num) => num as usize,
                                None => return,
                            };
                            log::debug!("Chose skill #{}", input);
                            match skills.get_mut(input) {
                                Some(skill) => {
                                    if skill.r#use().is_err() {
                                        term.messagebox("Skill still on cooldown");
                                    }
                                    break;
                                }
                                None => term.messagebox("Number out of bounds"),
                            }
                        }
                    }
                    GameAction::AddStatus => {
                        if let Some(status) = term.choose_status() {
                            log::debug!(
                                "Adding status {:?} for {}, type: {:?}",
                                status.status_type,
                                status.duration,
                                status.status_cooldown_type
                            );

                            get_player_mut!(players, id).add_status(status);
                        }
                    }
                    GameAction::DrainStatusAttacking => {
                        get_player_mut!(players, id).drain_status(StatusCooldownType::Attacking)
                    }
                    GameAction::DrainStatusAttacked => {
                        get_player_mut!(players, id).drain_status(StatusCooldownType::Attacked)
                    }
                    GameAction::ClearStatuses => get_player_mut!(players, id).statuses.clear(),
                    GameAction::ResetSkillsCD => {
                        log::debug!(
                            "Resetting all skill cd for {}",
                            get_player!(players, id).name
                        );
                        get_player_mut!(players, id)
                            .skills
                            .iter_mut()
                            .for_each(|skill| skill.available_after = 0);
                    }
                    GameAction::ManageMoney => get_player_mut!(players, id).manage_money(term),
                    GameAction::MakeTurn => {
                        get_player_mut!(players, id).turn();
                        break;
                    }
                    GameAction::SkipTurn => break,
                    GameAction::NextPlayerPick => {
                        log::debug!("Pending a next player change");
                        next_player = NextPlayerState::Pending;
                        continue 'game;
                    }
                    GameAction::Quit => break 'game,
                }
            }
        }
    }

    log::debug!("Exiting the game...");
}

fn character_menu(term: &Term, players: &mut Players) {
    log::debug!("In the character menu...");
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
                let biggest_id = if let Some(num) = players.keys().max() {
                    num + 1
                } else {
                    0
                };
                log::debug!("Adding a new player with id #{}", biggest_id);
                players.insert(biggest_id, Player::default());
                edit_player(term, players, biggest_id);
                // TODO: find out which pos the new player has in the list
                //last_selected = Some(id);
                last_selected = None;
            }
            CharacterMenuAction::Edit(num) => {
                log::debug!("Editing player #{}", num);
                edit_player(term, players, num);
                last_selected = Some(num);
            }
            // TODO: remove skills
            CharacterMenuAction::Delete(num) => {
                log::debug!("Confirming deletion of player #{}", num);
                if term.messagebox_yn("Are you sure?") {
                    log::debug!("Deleting #{}", num);
                    // TODO: fix logic. Use the actual id instead of order id
                    players.remove(num);
                    last_selected = None;
                } else {
                    log::debug!("Not confirmed");
                }
            }
            CharacterMenuAction::Quit { .. } => {
                log::debug!("Closing the character menu");
                break;
            }
            CharacterMenuAction::Editing { .. } | CharacterMenuAction::DoneEditing => {
                log::error!("How did we even get here??? CMA::Editing or CMA::DoneEditing were somehow returned from the character menu not in editing mode. Something went terribly wrong...");
                unreachable!();
            }
        }
    }

    log::debug!("Exiting the character menu...");
}

fn edit_player(term: &Term, players: &mut Players, id: usize) {
    log::debug!("Editing player #{}", id);
    let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
    loop {
        // init fields if they don't exist
        match selected_field {
            PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
                let player = get_player_mut!(players, id);
                if player.skills.get(skill_id).is_none() {
                    log::debug!("Going to modify a skill but it doesn't yet exist. Creating...");
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
                let player = get_player_mut!(players, id);
                match selected_field {
                    PlayerField::Name => {
                        log::debug!(
                            "Editing player #{} name: from {} to {}",
                            id,
                            player.name,
                            buffer
                        );
                        let _ = std::mem::replace(&mut player.name, buffer);

                        // TODO: modify inplace
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                    // TODO: maybe try to integrate stat id together with selected id in the enum?
                    PlayerField::Stat(selected) => {
                        let stat_id = {
                            let stat_list = STAT_LIST.lock().unwrap();
                            let vec = stat_list.as_vec();
                            *vec.get(selected).unwrap().0
                        };

                        if let Ok(buffer) = buffer
                            .parse::<i32>()
                            //.map_err(|e| log::error!("Error parsing new {:?} value: {}", stat, e))
                            .map_err(|e| {
                                log::error!("Error parsing new stat #{} value: {}", stat_id, e)
                            })
                        {
                            log::debug!(
                                //"Chaning player #{} stat {:?}: from {} to {}",
                                "Chaning player #{} stat #{} to {}",
                                id,
                                //stat,
                                stat_id,
                                buffer
                            );
                            player.stats.set(stat_id, buffer);
                            selected_field = match field_offset.unwrap_or(1) {
                                1 => selected_field.next(),
                                -1 => selected_field.prev(),
                                _ => selected_field,
                            }
                        }
                    }
                    PlayerField::SkillName(skill_id) => {
                        let skill_name = &mut player.skills[skill_id].name;
                        log::debug!(
                            "Changing player #{}'s skill #{}'s name: from {} to {}",
                            id,
                            skill_id,
                            skill_name,
                            buffer
                        );
                        let _ = std::mem::replace(skill_name, buffer);
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                    PlayerField::SkillCD(skill_id) => {
                        if let Ok(buffer) = buffer.parse::<u32>().map_err(|e| {
                            log::error!("Error parsing new skill #{} CD value: {}", skill_id, e)
                        }) {
                            let skill_cd = &mut player.skills[skill_id].cooldown;
                            log::debug!(
                                "Changing player #{}'s skill #{}'s CD: from {} to {}",
                                id,
                                skill_id,
                                skill_cd,
                                buffer
                            );
                            player.skills[skill_id].cooldown = buffer;
                        }
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                }
            }
            // TODO: properly check for empty buffer in player and skill names
            Some(CharacterMenuAction::DoneEditing) => {
                let player = get_player_mut!(players, id);
                log::debug!("Done editing {}", player.name);
                if let Some(skill) = player.skills.last() {
                    if skill.name.is_empty() {
                        log::debug!("Last skill's name is empty. Removing...");
                        player.skills.pop();
                    }
                }
                break;
            }
            _ => {
                log::error!("This should have never been reached. Somehow the CM in editing mode return an action other than CMA::Editing or CMD::DoneEditing");
                unreachable!();
            }
        }
    }

    log::debug!("Exiting out of the character menu...");
}
