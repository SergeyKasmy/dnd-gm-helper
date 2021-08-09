use anyhow::Result;
//pub mod action_enums;
mod action_enums;
mod entity_list;
mod player;
//pub mod player_field;
mod id;
mod player_field;
mod skill;
mod stats;
mod status;
mod term;

use action_enums::{CharacterMenuAction, GameAction, MainMenuAction};
use crossterm::event::KeyCode;
use entity_list::EntityList;
use id::Uid;
use once_cell::sync::Lazy;
use player::{Player, Players};
use player_field::PlayerField;
use serde::Deserialize;
use serde::Serialize;
use skill::Skill;
use stats::StatList;
use status::StatusCooldownType;
use std::collections::HashMap;
use std::sync::Mutex;
use term::list_state_ext::ListStateExt;
use term::{CharacterMenuMode, Term};
use tui::widgets::ListState;

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

#[derive(Serialize, Deserialize, Debug, Default)]
struct GameState {
	players: Players,
	order: Vec<Uid>,
}

macro_rules! get_player {
	($players:ident, $i:expr) => {
		$players
			.get($i)
			.ok_or(anyhow::Error::msg("Player not found"))
			// TODO: remove double errors
			.map_err(|e| log::error!("{} is not a valid id: {}", $i, e))
			// TODO: do something about the unwrap
			.unwrap()
	};
}

macro_rules! get_player_mut {
	($players:ident, $i:expr) => {
		$players
			.get_mut($i)
			.ok_or("Player not found")
			.map_err(|e| log::error!("{} is not a valid id: {}", $i, e))
			.unwrap()
	};
}

pub fn run() -> Result<()> {
	use std::panic;

	log::debug!("Starting...");
	log_panics::init();
	// TODO: do something about it
	if let Err(e) = panic::catch_unwind(start) {
		if let Ok(term) = Term::new() {
			let _ = term.messagebox("sowwy! OwO the pwogwam cwashed! 🥺 pwease d-don't bwame the d-devewopew, òωó he's d-doing his best!");
		}
		panic::resume_unwind(e);
	}
	Ok(())
}

fn start() -> Result<()> {
	let term = Term::new()?;
	let mut games: Vec<(String, GameState)> = Vec::new();

	let file_contents = std::fs::read_to_string("games.json");
	if let Ok(json) = file_contents.map_err(|e| log::info!("games.json could not be read: {}", e)) {
		match serde_json::from_str(&json) {
			Ok(data) => {
				log::debug!("Read from the db: {:#?}", data);
				games = data;
			}
			Err(e) => {
				// TODO: convert old format with Vec to the new with HashMap
				log::error!("The database is corrupted: {}", e);
				if term.messagebox_yn("The database is corrupted. Continue?")? {
					let db_bak = format!(
						"games.json.bak-{}",
						std::time::SystemTime::now()
							.duration_since(std::time::UNIX_EPOCH)?
							.as_secs()
					);
					log::info!("Coping the old corrupted db to {}", db_bak);
					let _ = std::fs::copy("games.json", db_bak)
						.map_err(|e| log::error!("Error copying: {}", e));
				} else {
					return Err(e.into());
				}
			}
		}
	}

	// sort games by name
	games.sort_by(|(a, _), (b, _)| a.cmp(b));

	let game_num = {
		let mut options = games
			.iter()
			.map(|(name, _)| name.as_str())
			.collect::<Vec<&str>>();
		options.push("Add...");
		loop {
			match term.messagebox_with_options("Choose the game", &options, true)? {
				Some(num) => {
					if num >= games.len().into() {
						let name =
							term.messagebox_with_input_field("Enter the name of the new game")?;
						games.push((name, GameState::default()));
					}
					break num;
				}
				None => return Ok(()),
			}
		}
	};

	let mut state = &mut games
		.get_mut(*game_num)
		.ok_or(anyhow::Error::msg("Game not found"))?
		.1;

	if !state.players.is_empty() && state.order.is_empty() {
		state.order = state.players.sort_ids().iter().map(|id| *id).collect();
	}

	loop {
		match term.draw_main_menu()? {
			MainMenuAction::Play => {
				if state.players.is_empty() {
					term.messagebox(
						"Can't start the game with no players. Try again after you add some",
					)?;
					continue;
				}
				if state.order.is_empty() {
					term.messagebox("There are no player in the so-called \"Player Order\". Who's gonna play the game if there is no order of players?")?;
					continue;
				}
				game_start(&term, &mut state.players, &state.order)?;
			}
			MainMenuAction::Edit => character_menu(&term, &mut state.players)?,
			MainMenuAction::ReorderPlayers => {
				if state.players.is_empty() {
					term.messagebox(
						"Can't reorder when there are no players. Try again after you add some",
					)?;
					continue;
				}
				state.order = reorder_players(&term, &state.order, &mut state.players)?
			}
			MainMenuAction::Quit => break,
		}
	}

	log::debug!("Saving game data to the db");
	std::fs::write("games.json", serde_json::to_string(&games)?).map_err(|e| {
		log::error!("Error saving game data to the db: {}", e);
		e
	})?;

	log::debug!("Exiting...");
	Ok(())
}

fn game_start(term: &Term, players: &mut Players, player_order: &[Uid]) -> Result<()> {
	log::debug!("In the game menu...");
	enum NextPlayerState {
		Default,
		Pending,
		Picked(*const Player),
	}
	let mut next_player = NextPlayerState::Default;

	// TODO: do this only if player_order is empty
	'game: loop {
		if let NextPlayerState::Pending = next_player {
			log::debug!("Pending a next player change.");
			if let Some(picked_player) = term.pick_player(players)? {
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
				match term.draw_game(get_player!(players, id))? {
					// TODO: combine lesser used options into a menu
					// TODO: use skills on others -> adds status
					// TODO: rename "Drain status" to "Got hit"/"Hit mob"
					GameAction::UseSkill => {
						// TODO: rework
						let skills = &mut get_player_mut!(players, id).skills;
						log::debug!("Choosing a skill to use");
						loop {
							let input = match term.choose_skill(skills)? {
								Some(num) => num,
								None => continue,
							};
							log::debug!("Chose skill #{}", input);
							match skills.get_mut(*input) {
								Some(skill) => {
									if skill.r#use().is_err() {
										term.messagebox("Skill still on cooldown")?;
									}
									break;
								}
								None => term.messagebox("Number out of bounds")?,
							}
						}
					}
					GameAction::AddStatus => {
						if let Some(status) = term.choose_status()? {
							log::debug!(
								"Adding status {:?} for {}, type: {:?}",
								status.status_type,
								status.duration,
								status.status_cooldown_type
							);

							get_player_mut!(players, id).add_status(status);
						}
					}
					GameAction::DrainStatus(StatusCooldownType::Normal) => unreachable!(),
					GameAction::DrainStatus(StatusCooldownType::OnAttacking) => {
						get_player_mut!(players, id)
							.drain_status_by_type(StatusCooldownType::OnAttacking)
					}
					GameAction::DrainStatus(StatusCooldownType::OnGettingAttacked) => {
						get_player_mut!(players, id)
							.drain_status_by_type(StatusCooldownType::OnGettingAttacked)
					}
					GameAction::DrainStatus(StatusCooldownType::Manual) => {
						log::debug!("Choosing which manual status to drain");
						let statuses = &get_player!(players, id).statuses;
						let manual_statuses = statuses
							.sort_ids()
							.iter()
							.filter_map(|x| {
								if get_player!(players, id)
									.statuses
									.get(*x)?
									.status_cooldown_type == StatusCooldownType::Manual
								{
									Some(*x)
								} else {
									None
								}
							})
							.collect::<Vec<Uid>>();
						let manual_statuses_list = manual_statuses
							.iter()
							.map(|&x| {
								format!(
									"{:?}, {} left",
									statuses.get(x).unwrap().status_type,
									statuses.get(x).unwrap().duration
								)
							})
							.collect::<Vec<String>>();
						if let Some(num) = term.messagebox_with_options(
							"Pick status",
							&manual_statuses_list,
							true,
						)? {
							get_player_mut!(players, id).statuses.drain_by_id(
								*manual_statuses
									.get(*num)
									.ok_or(anyhow::Error::msg("Couldn't drain manual status"))?,
							)?;
						}
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
							.for_each(|skill| skill.cooldown_left = 0);
					}
					GameAction::ManageMoney => {
						let diff = term.get_money_amount()?;
						get_player_mut!(players, id).manage_money(diff);
					}
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
	Ok(())
}

fn character_menu(term: &Term, players: &mut Players) -> Result<()> {
	log::debug!("In the character menu...");
	let mut last_selected: Option<Uid> = None;
	loop {
		match term.draw_character_menu(
			CharacterMenuMode::View {
				selected: last_selected,
			},
			players,
		)? {
			CharacterMenuAction::Add => {
				let id = players.push(Player::default());
				log::debug!("Added a new player with id #{}", id);
				edit_player(term, players, id)?;
				// TODO: find out which pos the new player has in the list
				//last_selected = Some(id);
				last_selected = None;
			}
			CharacterMenuAction::Edit(num) => {
				log::debug!("Editing player #{}", num);
				edit_player(term, players, num)?;
				last_selected = Some(num);
			}
			// TODO: remove skills
			CharacterMenuAction::Delete(num) => {
				log::debug!("Confirming deletion of player #{}", num);
				if term.messagebox_yn("Are you sure?")? {
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
	Ok(())
}

fn edit_player(term: &Term, players: &mut Players, id: Uid) -> Result<()> {
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
		)? {
			CharacterMenuAction::Editing {
				buffer,
				field_offset,
			} => {
				let player = get_player_mut!(players, id);
				match selected_field {
					PlayerField::Name => {
						log::debug!(
							"Editing player #{} name: from {} to {}",
							id,
							player.name,
							buffer
						);
						if !buffer.is_empty() {
							let _ = std::mem::replace(&mut player.name, buffer);

							// TODO: modify inplace
							selected_field = match field_offset.unwrap_or(1) {
								1 => selected_field.next(),
								-1 => selected_field.prev(),
								_ => selected_field,
							}
						}
					}
					// TODO: maybe try to integrate stat id together with selected id in the enum?
					PlayerField::Stat(selected) => {
						let stat_id = {
							let stat_list = STAT_LIST.lock().unwrap();
							let vec = stat_list.as_vec();
							*vec.get(selected)
								.ok_or(anyhow::Error::msg("Player not found"))?
								.0
						};

						if let Ok(buffer) = buffer
							.parse::<i32>()
							//.map_err(|e| log::error!("Error parsing new {:?} value: {}", stat, e))
							.map_err(|e| {
								log::error!("Error parsing new stat #{} value: {}", stat_id, e)
							}) {
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
			CharacterMenuAction::DoneEditing => {
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
	Ok(())
}

fn reorder_players(
	term: &Term,
	old_player_order: &[Uid],
	players: &mut Players,
) -> Result<Vec<Uid>> {
	let mut player_list: Vec<(Uid, &str)> = old_player_order
		.iter()
		.map(|&id| (id, players.get(id).unwrap().name.as_str()))
		.collect();
	log::debug!("Old player order with names: {:#?}", player_list);
	let mut state = ListState::default();
	loop {
		let mut options: Vec<&str> = player_list.iter().map(|(_, name)| *name).collect();
		// TODO: add an option to add a removed player without resetting
		options.push("Reset");
		match term.messagebox_with_options("Choose which player to move", &options, true)? {
			Some(num) => {
				// Reset is the last option, not an actual player name
				if num == (options.len() - 1).into() {
					player_list = players
						.sort_ids()
						.iter()
						.map(|&id| (id, players.get(id).unwrap().name.as_str()))
						.collect();
					continue;
				}
				state.select_onum(Some(num));
				loop {
					// TODO: dedup
					let name_list: Vec<&str> = player_list.iter().map(|(_, name)| *name).collect();
					log::debug!("Moving player #{}", state.selected().unwrap());
					// TODO: move this inside Term. the controller should be Ui agnostic
					match term.messagebox_with_options_immediate(
						"Use arrows to move the player | D to remove them entirely",
						&name_list,
						state.selected_onum(),
						true,
					)? {
						// TODO: add more checks for unwrap()
						KeyCode::Down => {
							let selected = state.selected().unwrap();
							if selected + 1 >= player_list.len() {
								continue;
							}
							log::debug!("Old player order in the Vec: {:#?}", player_list);
							player_list.swap(selected, selected + 1);
							state.next(player_list.len());
						}
						KeyCode::Up => {
							let selected = state.selected().unwrap();
							if let None = selected.checked_sub(1) {
								continue;
							}
							log::debug!("Old player order in the Vec: {:#?}", player_list);
							player_list.swap(selected, selected - 1);
							state.prev(player_list.len());
						}
						KeyCode::Char('d') => {
							let selected = state.selected().unwrap();
							player_list.remove(selected);
							break;
						}
						KeyCode::Enter | KeyCode::Esc => {
							break;
						}
						_ => (),
					}
				}
			}
			None => break,
		}
	}

	Ok(player_list.into_iter().map(|(id, _)| id).collect())
}
