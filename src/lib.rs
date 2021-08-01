// TODO: add logging
// TODO: add force crash with vars and bt
mod term;
mod player;
mod skill;
mod status;
mod stat;

use term::{
    action_enums::{CharacterMenuAction, GameAction, MainMenuAction},
    player_field::PlayerField,
    CharacterMenuMode, Term,
};
use player::Player;
use skill::Skill;
use status::{Status, StatusCooldownType};
use stat::{Stats, StatType};

fn game_start(term: &Term, players: &mut Vec<Player>) {
    enum NextPlayerState {
        Default,
        Pending,
        Picked(*const Player),
    }
    let mut next_player = NextPlayerState::Default;
    'game: loop {
        if let NextPlayerState::Pending = next_player {
            if let Some(picked_player) = term.pick_player(players) {
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
                match term.draw_game(player) {
                    // TODO: reorder players + sorting
                    // TODO: combine lesser used options into a menu
                    // TODO: use skills on others -> adds status
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

fn choose_skill_and_use(term: &Term, skills: &mut Vec<Skill>) {
    loop {
        let input = match term.choose_skill(skills) {
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

fn add_status(term: &Term, statuses: &mut Vec<Status>) {
    if let Some(status) = term.choose_status() {
        statuses.push(status);
    }
}

fn manage_money(term: &Term, player: &mut Player) {
    player.money += term.get_money_amount();
}

pub fn run() {
    let term = Term::new();
    let mut players = vec![];
    let file_contents = std::fs::read_to_string("players.json");
    if let Ok(json) = file_contents {
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

    loop {
        match term.draw_main_menu() {
            MainMenuAction::Play => game_start(&term, &mut players),
            MainMenuAction::Edit => character_menu(&term, &mut players),
            MainMenuAction::Quit => break,
        }
    }

    std::fs::write("players.json", serde_json::to_string(&players).unwrap()).unwrap();
}

fn character_menu(term: &Term, players: &mut Vec<Player>) {
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
            // TODO: remove skills
            CharacterMenuAction::Delete(num) => {
                if term.messagebox_yn("Are you sure?") {
                    players.remove(num);
                    last_selected = num.checked_sub(1);
                }
            }
            CharacterMenuAction::Quit { .. } => break,
            CharacterMenuAction::Editing { .. } | CharacterMenuAction::DoneEditing => {
                unreachable!()
            }
        }
    }
}

fn edit_player(term: &Term, players: &mut Vec<Player>, id: usize) {
    let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
    loop {
        //
        // init fields if they don't exist
        match selected_field {
            PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
                let player = players.get_mut(id).unwrap();
                if player.skills.get(skill_id).is_none() {
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

                        if let Ok(buffer) = buffer.parse::<i64>() {
                                *current_stat = buffer;
                                selected_field = match field_offset.unwrap_or(1) {
                                    1 => selected_field.next(),
                                    -1 => selected_field.prev(),
                                    _ => selected_field,
                                }
                        }
                    }
                    PlayerField::SkillName(skill_id) => {
                        let _ = std::mem::replace(&mut player.skills[skill_id].name, buffer);
                        selected_field = match field_offset.unwrap_or(1) {
                            1 => selected_field.next(),
                            -1 => selected_field.prev(),
                            _ => selected_field,
                        }
                    }
                    PlayerField::SkillCD(skill_id) => {
                        if let Ok(buffer) = buffer.parse::<u32>() {
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
                if let Some(player) = players.get_mut(id) {
                    if let Some(skill) = player.skills.last() {
                        if skill.name.is_empty() {
                            player.skills.pop();
                        }
                    }
                }
                return;
            }
            _ => unreachable!(),
        }
    }
}
