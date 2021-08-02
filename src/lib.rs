// TODO: add force crash with vars and bt
mod player;
mod skill;
mod stat;
mod status;
mod term;

use player::Player;
use skill::Skill;
use stat::{StatType, Stats};
use status::{Status, StatusCooldownType};
use term::{
    action_enums::{CharacterMenuAction, GameAction, MainMenuAction},
    player_field::PlayerField,
    CharacterMenuMode, Term,
};

fn game_start(term: &Term, players: &mut Vec<Player>) {
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

        for player in players.iter_mut() {
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
                        log::debug!("Resetting all skill cd for {}", player.name);
                        player
                            .skills
                            .iter_mut()
                            .for_each(|skill| skill.available_after = 0);
                    }
                    GameAction::ManageMoney => manage_money(term, player),
                    GameAction::MakeTurn => {
                        turn(player);
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

fn use_skill(skill: &mut Skill) {
    log::debug!("Using skill {}", skill.name);
    skill.available_after = skill.cooldown;
}

fn drain_status(player: &mut Player, status_type: StatusCooldownType) {
    log::debug!(
        "Draining statuses for {} with type {:?}",
        player.name,
        status_type
    );
    // decrease all statuses duration with the status cooldown type provided
    player.statuses.iter_mut().for_each(|status| {
        if status.status_cooldown_type == status_type && status.duration > 0 {
            log::debug!("Drained {:?}", status.status_type);
            status.duration -= 1
        }
    });
    // remove all statuses that have run out = retain all statuses that haven't yet run out
    player.statuses.retain(|status| status.duration > 0);
}

fn choose_skill_and_use(term: &Term, skills: &mut Vec<Skill>) {
    log::debug!("Choosing a skill to use");
    loop {
        let input = match term.choose_skill(skills) {
            Some(num) => num as usize,
            None => return,
        };
        log::debug!("Chose skill #{}", input);
        match skills.get_mut(input) {
            Some(skill) => {
                if skill.available_after == 0 {
                    use_skill(skill);
                } else {
                    log::debug!("Skill {} is still on cooldown", skill.name);
                    term.messagebox("Skill still on cooldown");
                }
                break;
            }
            None => term.messagebox("Number out of bounds"),
        }
    }
}

fn turn(player: &mut Player) {
    log::debug!("{}'s turn has ended", player.name);
    player.skills.iter_mut().for_each(|skill| {
        if skill.available_after > 0 {
            skill.available_after -= 1
        }
    });
    drain_status(player, StatusCooldownType::Normal);
}

fn add_status(term: &Term, statuses: &mut Vec<Status>) {
    if let Some(status) = term.choose_status() {
        log::debug!("Adding status {:?} for {}, type: {:?}", status.status_type, status.duration, status.status_cooldown_type);
        statuses.push(status);
    }
}

fn manage_money(term: &Term, player: &mut Player) {
    let diff = term.get_money_amount();
    log::debug!("Adding {} money to Player {}", diff, player.name);
    player.money += diff;
}

pub fn run() {
    log::debug!("Starting...");
    let term = Term::new();
    let mut players = vec![];
    let file_contents = std::fs::read_to_string("players.json");
    if let Ok(json) = file_contents.map_err(|e| log::info!("players.json could not be read: {}", e)) {
        match serde_json::from_str(&json) {
            Ok(data) => {
                log::debug!("Read from the db: {:#?}", data);
                players = data;
            }
            Err(_) => {
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
    let _ = std::fs::write("players.json", serde_json::to_string(&players).unwrap()).map_err(|e| log::error!("Error saving to the db: {}", e));
    log::debug!("Exiting...");
}

fn character_menu(term: &Term, players: &mut Vec<Player>) {
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
                players.push(Player::default());
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
                log::debug!("Confirming deleting of player #{}", num);
                if term.messagebox_yn("Are you sure?") {
                    log::debug!("Deleing #{}", num);
                    players.remove(num);
                    last_selected = num.checked_sub(1);
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

// TODO: mb use just Player instead of Vec<Player>?
fn edit_player(term: &Term, players: &mut Vec<Player>, id: usize) {
    log::debug!("Editing player #{}", id);
    let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
    loop {
        // init fields if they don't exist
        match selected_field {
            PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
                let player = players
                    .get_mut(id)
                    .ok_or(0)
                    .map_err(|_| log::error!("Player #{} wasn't found in the vector", id))
                    .unwrap();
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
                let player = players.get_mut(id).unwrap();
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
                log::debug!("Done editing player #{}", id);
                if let Some(player) = players.get_mut(id) {
                    if let Some(skill) = player.skills.last() {
                        if skill.name.is_empty() {
                            log::debug!("Last skill's name is empty. Removing...");
                            player.skills.pop();
                        }
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

    log::debug!("Finished editing player #{}", id);
}
