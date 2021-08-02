// TODO: add force crash with vars and bt
mod player;
mod skill;
mod stat;
mod status;
mod term;

use player::{Player, Players};
use skill::Skill;
use stat::{StatType, Stats};
use status::{Status, StatusCooldownType};
use std::collections::HashMap;
use term::{
    action_enums::{CharacterMenuAction, GameAction, MainMenuAction},
    player_field::PlayerField,
    CharacterMenuMode, Term,
};

fn game_start(term: &Term, players: &mut Players) {
    log::debug!("In the game menu...");
    enum NextPlayerState {
        Default,
        Pending,
        Picked(*const Player),
    }
    let mut next_player = NextPlayerState::Default;

    'game: loop {
        if let NextPlayerState::Pending = next_player {
            log::debug!("Pending a next player change.");
            if let Some(picked_player) = term.pick_player(players) {
                log::debug!("Picked next player: {}", picked_player.name);
                next_player = NextPlayerState::Picked(picked_player);
            }
        }

        for (_, player) in players.iter_mut() {
            if let NextPlayerState::Picked(next_player) = next_player {
                if !std::ptr::eq(next_player, player) {
                    log::debug!("Skipping player {}", player.name);
                    continue;
                }
            }
            log::debug!("Current turn: {}", player.name);
            loop {
                match term.draw_game(player) {
                    // TODO: reorder players + sorting
                    // TODO: combine lesser used options into a menu
                    // TODO: use skills on others -> adds status
                    // TODO: rename "Drain status" to "Got hit"/"Hit mob"
                    GameAction::UseSkill => {
                        // TODO: rework
                        let skills = &mut player.skills;
                        log::debug!("Choosing a skill to use");
                        loop {
                            let input = match term.choose_skill(skills) {
                                Some(num) => num as usize,
                                None => return,
                            };
                            log::debug!("Chose skill #{}", input);
                            match skills.get_mut(input) {
                                Some(skill) => {
                                    if let Err(_) = skill.r#use() {
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

                            player.add_status(status);
                        }
                    }
                    GameAction::DrainStatusAttacking => {
                        player.drain_status(StatusCooldownType::Attacking)
                    }
                    GameAction::DrainStatusAttacked => {
                        player.drain_status(StatusCooldownType::Attacked)
                    }
                    GameAction::ClearStatuses => player.statuses.clear(),
                    GameAction::ResetSkillsCD => {
                        log::debug!("Resetting all skill cd for {}", player.name);
                        player
                            .skills
                            .iter_mut()
                            .for_each(|skill| skill.available_after = 0);
                    }
                    GameAction::ManageMoney => player.manage_money(term),
                    GameAction::MakeTurn => {
                        player.turn();
                        break;
                    }
                    GameAction::SkipTurn => break,
                    GameAction::NextPlayerPick => {
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

pub fn run() {
    log::debug!("Starting...");
    let term = Term::new();
    let mut players: Players = HashMap::new();
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
                log::debug!("Adding a new player");
                let biggest_id = if let Some(num) = players.keys().max() {
                    num + 1
                } else {
                    0
                };
                players.insert(biggest_id, Player::default());
                let id = players.len() - 1;
                edit_player(term, players, id);
                last_selected = Some(id);
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
                    players.remove(&num);
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
    macro_rules! get_player {
        () => {
            players
                .get_mut(&id)
                .ok_or(0)
                .map_err(|_| log::error!("Player #{} wasn't found in the hashmap", id))
                .unwrap()
        };
    }
    log::debug!("Editing player #{}", id);
    let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
    loop {
        // init fields if they don't exist
        match selected_field {
            PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
                let player = get_player!();
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
                let player = get_player!();
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
                    PlayerField::Stat(stat) => {
                        let current_stat = match stat {
                            StatType::Strength => &mut player.stats.strength,
                            StatType::Dexterity => &mut player.stats.dexterity,
                            StatType::Poise => &mut player.stats.poise,
                            StatType::Wisdom => &mut player.stats.wisdom,
                            StatType::Intelligence => &mut player.stats.intelligence,
                            StatType::Charisma => &mut player.stats.charisma,
                        };

                        if let Ok(buffer) = buffer
                            .parse::<i64>()
                            .map_err(|e| log::error!("Error parsing new {:?} value: {}", stat, e))
                        {
                            log::debug!(
                                "Chaning player #{} stat {:?}: from {} to {}",
                                id,
                                stat,
                                current_stat,
                                buffer
                            );
                            *current_stat = buffer;
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
                let player = get_player!();
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

fn sort_player_list(players: &Players) -> Vec<(&usize, &Player)> {
    log::debug!("Sorting player list");
    let mut players_sorted = players.iter().collect::<Vec<(&usize, &Player)>>();
    players_sorted.sort_unstable_by(|a, b| a.1.name.cmp(&b.1.name));
    players_sorted
}
