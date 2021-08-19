use crate::term::Term as Ui;
use anyhow::Result;
use dnd_gm_helper::side_effect::{SideEffectAffects, SideEffectType};
use dnd_gm_helper::{action_enums::EditorActionEditMode, skill::Skill};
use dnd_gm_helper::{
	action_enums::{
		EditorAction, EditorActionViewMode, GameAction, MainMenuAction, SettingsAction,
	},
	id::Uid,
	player::{Player, Players},
	server::Server,
	stats::StatList,
	status::{StatusCooldownType, StatusList},
};

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
	/*
	use std::panic;

	log::debug!("Starting...");
	log_panics::init();
	// TODO: do something about it
	if let Err(e) = panic::catch_unwind(start) {
		if let Ok(ui) = Ui::new() {
			let _ = ui.messagebox("sowwy! OwO the pwogwam cwashed! 🥺 pwease d-don't bwame the d-devewopew, òωó he's d-doing his best!");
		}
		panic::resume_unwind(e);
	}
	Ok(())
	*/

	let ui = Ui::new()?;

	let mut server = Server::new()?;

	let game_num = {
		let names = server.get_names();
		let mut options = Vec::new();
		options.clone_from(&names);
		options.push("Add...");
		loop {
			match ui.messagebox_with_options("Choose the game", &options, true)? {
				Some(num) => {
					if num >= names.len().into() {
						let name =
							ui.messagebox_with_input_field("Enter the name of the new game")?;
						server.add_game(name);
					}
					break num;
				}
				None => return Ok(()),
			}
		}
	};
	server.set_current_game_num(game_num);
	main_menu(&ui, &mut server)?;
	server.save()?;

	Ok(())
}

fn main_menu(ui: &Ui, server: &mut Server) -> Result<()> {
	let state = server.get_current_game_state().unwrap();
	loop {
		match ui.draw_main_menu()? {
			MainMenuAction::Play => {
				if state.players.is_empty() {
					ui.messagebox(
						"Can't start the game with no players. Try again after you add some",
					)?;
					continue;
				}
				if state.order.is_empty() {
					ui.messagebox("There are no player in the so-called \"Player Order\". Who's gonna play the game if there is no order of players?")?;
					continue;
				}
				game_start(
					ui,
					&mut state.players,
					&state.order,
					&state.stat_list,
					&state.status_list,
				)?;
			}
			MainMenuAction::EditPlayers => {
				character_menu(ui, &mut state.players, &state.stat_list, &state.status_list)?
			}
			MainMenuAction::ReorderPlayers => {
				if state.players.is_empty() {
					ui.messagebox(
						"Can't reorder when there are no players. Try again after you add some",
					)?;
					continue;
				}
				state.order = reorder_players(ui, &state.order, &mut state.players)?
			}
			MainMenuAction::Settings => match ui.draw_settings_menu()? {
				SettingsAction::EditStats => statlist_menu(ui, &mut state.stat_list)?,
				SettingsAction::EditStatuses => statuslist_menu(ui, &mut state.status_list)?,
				SettingsAction::GoBack => continue,
			},
			MainMenuAction::Quit => break,
		}
	}
	Ok(())
}

fn game_start(
	ui: &Ui,
	players: &mut Players,
	player_order: &[Uid],
	stat_list: &StatList,
	status_list: &StatusList,
) -> Result<()> {
	log::debug!("In the game menu...");
	enum NextPlayerState {
		Default,
		Pending,
		Picked(*const Player),
	}
	assert!(!player_order.is_empty());

	let mut next_player = NextPlayerState::Default;
	'game: loop {
		if let NextPlayerState::Pending = next_player {
			log::debug!("Pending a next player change.");
			if let Some(picked_player) = ui.pick_player(players, None)? {
				log::debug!("Picked next player: {}", picked_player.name);
				next_player = NextPlayerState::Picked(picked_player);
			}
		}

		for &id in player_order.iter() {
			if let NextPlayerState::Picked(next_player_ptr) = next_player {
				let player = get_player!(players, id);
				if !std::ptr::eq(next_player_ptr, player) {
					log::debug!("Skipping player {}", player.name);
					continue;
				}
				next_player = NextPlayerState::Default;
			}
			log::debug!("Current turn: {} #{}", get_player!(players, id).name, id);
			loop {
				match ui.draw_game(get_player!(players, id), stat_list)? {
					// TODO: combine lesser used options into a menu
					// TODO: use skills on others -> adds status
					// TODO: rename "Drain status" to "Got hit"/"Hit mob"
					GameAction::UseSkill => {
						let input = match ui.choose_skill(&get_player_mut!(players, id).skills)? {
							Some(num) => num,
							None => continue,
						};
						log::debug!("Choose skill #{}", input);
						match get_player_mut!(players, id).skills.get_mut(*input) {
							Some(skill) => {
								if skill.r#use().is_err() {
									ui.messagebox("Skill still on cooldown")?;
									continue;
								}
							}
							None => {
								ui.messagebox("Number out of bounds")?;
								continue;
							}
						}
						if let Some(side_effect) = &get_player!(players, id)
							.skills
							.get(*input)
							.unwrap()
							.side_effect
						{
							match &side_effect.r#type {
								SideEffectType::AddsStatus(status) => {
									ui.messagebox("This skill has an \"Adds status\" side effect")?;
									// TODO: avoid cloning
									let affects = side_effect.affects.clone();
									let status = status.clone();
									if let SideEffectAffects::Themselves | SideEffectAffects::Both =
										affects
									{
										ui.messagebox(format!(
											"Applying status {} to the player",
											status.status_type
										))?;
										get_player_mut!(players, id).add_status(status.clone())
									}
									if let SideEffectAffects::SomeoneElse
									| SideEffectAffects::Both = affects
									{
										ui.messagebox(format!(
											"Applying status {} to a different player",
											status.status_type
										))?;
										if let Some(target) = ui
											.pick_player(players, Some(id))?
											.map(|x| x.id.unwrap())
										{
											get_player_mut!(players, target)
												.add_status(status.clone());
										}
									}
								}
								SideEffectType::UsesSkill => {
									ui.messagebox("This skill has an \"Uses skill\" side effect. Choose a player and the skill to use")?;
									loop {
										if let Some(target) = ui
											.pick_player(players, Some(id))?
											.map(|x| x.id.unwrap())
										{
											let skill_names = get_player!(players, target)
												.skills
												.iter()
												.map(|x| x.name.as_str())
												.collect::<Vec<&str>>();
											if let Some(chosen_skill) = ui.messagebox_with_options(
												"Choose skill",
												&skill_names,
												true,
											)? {
												if get_player_mut!(players, target).skills
													[*chosen_skill]
													.r#use()
													.is_err()
												{
													// FIXME: may get stuck in a loop if all skills
													// are on cd. Do this somehow else
													ui.messagebox("Skill already on cooldown. Choose a different one")?;
													continue;
												} else {
													break;
												}
											}
										}
									}
								}
							}
						}
					}
					GameAction::AddStatus => {
						if let Some(status) = ui.choose_status(status_list)? {
							log::debug!(
								"Adding status {:?} for {}, type: {:?}",
								status.status_type,
								status.duration_left,
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
							.iter()
							.filter_map(|(&id, x)| {
								if x.status_cooldown_type == StatusCooldownType::Manual {
									Some(id)
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
									statuses.get(x).unwrap().duration_left
								)
							})
							.collect::<Vec<String>>();
						if let Some(num) =
							ui.messagebox_with_options("Pick status", &manual_statuses_list, true)?
						{
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
						let diff = ui.get_money_amount()?;
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

fn character_menu(
	ui: &Ui,
	players: &mut Players,
	stat_list: &StatList,
	status_list: &StatusList,
) -> Result<()> {
	loop {
		match ui.draw_character_menu(players, stat_list)? {
			EditorActionViewMode::Add => {
				//state.select(Some(player_names_list.len()));
				let id = players.push(Player::default());
				log::debug!("Added a new player with #{:?}", id);
				let added = ui.edit_player(players, id, stat_list, status_list)?;
				// TODO: find out which pos the new player has in the list
				//last_selected = Some(id);
                if let Some(added) = added {
                    players.insert(id, added);
                } else {
                    players.remove(id);
                }
			}
			EditorActionViewMode::Edit(num) => {
				log::debug!("Editing player #{:?}", num);
                let id = *players.get_by_index(num).unwrap().0;
				let edited = ui.edit_player(
					players,
                    id,
					stat_list,
					status_list,
				)?;
                if let Some(edited) = edited {
                    players.insert(id, edited);
                } else {
                    players.remove(id);
                }
			}
			EditorActionViewMode::Delete(num) => {
				log::debug!("Confirming deletion of player #{:?}", num);
				if ui.messagebox_yn("Are you sure?")? {
					log::debug!("Deleting #{:?}", num);
					//state.next(player_names_list.len() - 1);
					players.remove(*players.get_by_index(num).unwrap().0);
				} else {
					log::debug!("Not confirmed");
				}
			}
			EditorActionViewMode::Quit => {
				log::debug!("Closing the character menu");
				break;
			}
			EditorActionViewMode::Next | EditorActionViewMode::Prev => unreachable!(),
		}
	}

	Ok(())
}

fn reorder_players(ui: &Ui, old_player_order: &[Uid], players: &mut Players) -> Result<Vec<Uid>> {
    todo!();
    /*
	let mut player_list: IndexMap<Uid, &str> = old_player_order
		.iter()
		.map(|&id| (id, players.get(id).unwrap().name.as_str()))
		.collect();
	log::debug!("Old player order with names: {:#?}", player_list);
	let mut state = ListState::default();
	loop {
		let mut options: Vec<&str> = player_list.iter().map(|(_, name)| *name).collect();
		// TODO: add an option to add a removed player without resetting
		options.push("Reset");
		match ui.messagebox_with_options("Choose which player to move", &options, true)? {
			Some(num) => {
				// Reset is the last option, not an actual player name
				if num == (options.len() - 1).into() {
					player_list = players
						.iter()
						.map(|(id, pl)| (*id, pl.name.as_str()))
						.collect();
					continue;
				}
				state.select_onum(Some(num));
				loop {
					let name_list: Vec<&str> = player_list.iter().map(|(_, name)| *name).collect();
					log::debug!("Moving player #{}", state.selected().unwrap());
					// TODO: move this inside Ui. the controller should be Ui agnostic
					match ui.messagebox_with_options_immediate(
						"Use arrows to move the player | D to remove them entirely",
						&name_list,
						state.selected_onum(),
						true,
					)? {
						KeyCode::Down => {
							let selected = state.selected().unwrap();
							if selected + 1 >= player_list.len() {
								continue;
							}
							log::debug!("Old player order in the Vec: {:#?}", player_list);
							player_list.swap_indices(selected, selected + 1);
							state.next(player_list.len());
						}
						KeyCode::Up => {
							let selected = state.selected().unwrap();
							if let None = selected.checked_sub(1) {
								continue;
							}
							log::debug!("Old player order in the Vec: {:#?}", player_list);
							player_list.swap_indices(selected, selected - 1);
							state.prev(player_list.len());
						}
						KeyCode::Char('d') => {
							let selected = state.selected().unwrap();
							player_list.remove(&Uid(selected));
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
    */
}

fn statlist_menu(ui: &Ui, stat_list: &mut StatList) -> Result<()> {
    todo!();
    /*
	log::debug!("In the statlist menu...");
	// TODO: create a UI agnostic list state tracker
	// TODO: preselect the first
	let mut state = ListState::default();
	state.next(stat_list.len());
	loop {
		match ui.draw_editor(
			EditorMode::View {
				selected: state.selected_onum(),
			},
			Some("Stats"),
			&stat_list.get_names(),
			None::<fn(_) -> _>,
		)? {
			EditorAction::View(EditorActionViewMode::Add) => {
				log::debug!("Added a new status");
				stat_list.insert(ui.edit_setlist(
					stat_list,
					String::new(),
					stat_list.len().into(),
					Some("Stats"),
				)?);
				// TODO: find out which pos the new stat has in the list
				//last_selected = Some(id);
			}
			EditorAction::View(EditorActionViewMode::Edit) => {
				if let Some(num) = state.selected_onum() {
					log::debug!("Editing status #{:?}", num);
					// FIXME: avoid clonning
					let stat = stat_list
						.remove(&stat_list.get(num).unwrap().clone())
						.unwrap();
					stat_list.insert(ui.edit_setlist(stat_list, stat.1, stat.0, Some("Stats"))?);
				}
			}
			EditorAction::View(EditorActionViewMode::Delete) => {
				if let Some(num) = state.selected_onum() {
					log::debug!("Confirming deletion of stat #{:?}", num);
					if ui.messagebox_yn("Are you sure?")? {
						log::debug!("Deleting #{:?}", num);
						state.next(stat_list.len() - 1);
						let stat_name = stat_list.get(num).unwrap().to_string();
						stat_list.remove(&stat_name);
					} else {
						log::debug!("Not confirmed");
					}
				}
			}
			EditorAction::View(EditorActionViewMode::Next) => {
				state.next(stat_list.len());
			}
			EditorAction::View(EditorActionViewMode::Prev) => {
				state.prev(stat_list.len());
			}
			EditorAction::View(EditorActionViewMode::Quit) => {
				log::debug!("Closing the character menu");
				break;
			}
			EditorAction::Edit(_) => {
				log::error!("How did we even get here??? EditorAction::Edit was somehow returned from the editor not in editing mode. Something went terribly wrong...");
				unreachable!();
			}
		}
	}

	log::debug!("Exiting the character menu...");
	Ok(())
    */
}

fn statuslist_menu(ui: &Ui, status_list: &mut StatusList) -> Result<()> {
    todo!();
    /*
	log::debug!("In the statuslist menu...");
	// TODO: create a UI agnostic list state tracker
	// TODO: preselect the first
	let mut state = ListState::default();
	state.next(status_list.len());
	loop {
		let status_names_list = status_list.get_names();
		match ui.draw_editor(
			EditorMode::View {
				selected: state.selected_onum(),
			},
			Some("Statuses"),
			&status_names_list,
			None::<fn(_) -> _>,
		)? {
			EditorAction::View(EditorActionViewMode::Add) => {
				log::debug!("Added a new status");
				status_list.insert(ui.edit_setlist(
					status_list,
					String::new(),
					status_list.len().into(),
					Some("Statuses"),
				)?);
				// TODO: find out which pos the new stat has in the list
				//last_selected = Some(id);
			}
			EditorAction::View(EditorActionViewMode::Edit) => {
				if let Some(num) = state.selected_onum() {
					log::debug!("Editing status #{:?}", num);
					let status = status_list
						.remove(&status_list.get(num).unwrap().clone())
						.unwrap();
					status_list.insert(ui.edit_setlist(
						status_list,
						status.1,
						status.0,
						Some("Statuses"),
					)?);
				}
			}
			EditorAction::View(EditorActionViewMode::Delete) => {
				if let Some(num) = state.selected_onum() {
					log::debug!("Confirming deletion of status #{:?}", num);
					if ui.messagebox_yn("Are you sure?")? {
						log::debug!("Deleting #{:?}", num);
						state.next(status_names_list.len() - 1);
						let status_name = status_list.get(num).unwrap().to_string();
						status_list.remove(&status_name);
					} else {
						log::debug!("Not confirmed");
					}
				}
			}
			EditorAction::View(EditorActionViewMode::Next) => {
				state.next(status_names_list.len());
			}
			EditorAction::View(EditorActionViewMode::Prev) => {
				state.prev(status_names_list.len());
			}
			EditorAction::View(EditorActionViewMode::Quit) => {
				log::debug!("Closing the character menu");
				break;
			}
			EditorAction::Edit(_) => {
				log::error!("How did we even get here??? EditorAction::Edit was somehow returned from the editor not in editing mode. Something went terribly wrong...");
				unreachable!();
			}
		}
	}

	log::debug!("Exiting the character menu...");
	Ok(())
    */
}
