use crate::term::Term as Ui;
use crate::term::{list_state_ext::ListStateExt, EditorMode};
use anyhow::Result;
use crossterm::event::KeyCode;
use dnd_gm_helper::action::PlayerData;
use dnd_gm_helper::action::{
	ClientAction,
	ClientMessage::{self, *},
	ServerAction, ServerMessage,
};
use dnd_gm_helper::{
	action::{
		EditorAction, EditorActionEditMode, EditorActionViewMode, GameAction, MainMenuAction,
		SettingsAction,
	},
	id::Uid,
	player::{Player, Players},
	player_field::PlayerField,
	server::Server,
	skill::Skill,
	stats::StatList,
	status::{StatusCooldownType, StatusList},
};
use indexmap::IndexMap;
use std::borrow::Cow;
use std::sync::mpsc;
use std::thread;
use tui::widgets::ListState;

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

struct Client {
	ui: Ui,
	action_rx: mpsc::Receiver<ClientAction>,
	server_tx: mpsc::Sender<ServerAction>,
}

impl Client {
	fn new() -> Self {
		// client to server
		let (cts_tx, cts_rx) = mpsc::channel();
		// server to client
		let (stc_tx, stc_rx) = mpsc::channel();

		// start server
		// TODO: maybe create it somewhere else and not at the same time as the client?
		thread::spawn(move || {
			if let Ok(server) = Server::new(cts_rx, stc_tx) {
				server.start();
			}
		});

		Self {
			ui: Ui::new().unwrap(),
			server_tx: cts_tx,
			action_rx: stc_rx,
		}
	}

	fn send_to_server(&self, message: ClientMessage) -> Result<()> {
		self.server_tx
			.send(ServerAction::ClientMessage(0.into(), message))?;
		Ok(())
	}

	pub fn run(&self) -> Result<()> {
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
		let game_num = {
			let names: Vec<Cow<str>> = {
				self.send_to_server(RequestGameList)?;
				loop {
					match self.action_rx.recv() {
						Ok(ClientAction::ServerMessage(ServerMessage::GameList(list))) => {
							break list.into_iter().map(|x| Cow::Owned(x)).collect()
						}
						_ => todo!(),
					}
				}
			};
			let mut options = Vec::new();
			options.clone_from(&names);
			options.push(Cow::Borrowed("Add..."));
			loop {
				match self
					.ui
					.messagebox_with_options("Choose the game", &options, true)?
				{
					Some(num) => {
						if num >= names.len().into() {
							let name = self
								.ui
								.messagebox_with_input_field("Enter the name of the new game")?;
							self.send_to_server(AddNewGame(name));
						}
						break num;
					}
					None => return Ok(()),
				}
			}
		};
		self.send_to_server(ClientMessage::SetCurrentGame(game_num))?;
		self.main_menu()?;
		self.send_to_server(ClientMessage::Save)?;

		Ok(())
	}

	fn main_menu(&self) -> Result<()> {
		loop {
			match self.ui.draw_main_menu()? {
				MainMenuAction::Play => {
                    self.send_to_server(RequestPlayerData(PlayerData::IsEmpty(None)));
                    let is_empty = loop {
                        match self.action_rx.recv() {
                            Ok(ClientAction::ServerMessage(ServerMessage::PlayerData(PlayerData::IsEmpty(Some(is_empty))))) => break is_empty,
                            _ => (),
                        }
                    };
					if is_empty {
						self.ui.messagebox(
							"Can't start the game with no players. Try again after you add some",
						)?;
						continue;
					}
// ----------------------------------------------------------------------------------------------------------------------------------------------------------
//  STOPPED REWORKING HERE, DOESN'T WORK FROM HERE ON OUT. OLD CODE
// ----------------------------------------------------------------------------------------------------------------------------------------------------------
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
			// FIXME: crashes
			if let NextPlayerState::Pending = next_player {
				log::debug!("Pending a next player change.");
				if let Some(picked_player) = ui.pick_player(players)? {
					log::debug!("Picked next player: {}", picked_player.name);
					next_player = NextPlayerState::Picked(picked_player);
				}
			}

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
					match ui.draw_game(get_player!(players, id), stat_list, status_list)? {
						// TODO: combine lesser used options into a menu
						// TODO: use skills on others -> adds status
						// TODO: rename "Drain status" to "Got hit"/"Hit mob"
						GameAction::UseSkill => {
							let skills = &mut get_player_mut!(players, id).skills;
							log::debug!("Choosing a skill to use");
							loop {
								let input = match ui.choose_skill(skills)? {
									Some(num) => num,
									None => continue,
								};
								log::debug!("Chose skill #{}", input);
								match skills.get_mut(*input) {
									Some(skill) => {
										if skill.r#use().is_err() {
											ui.messagebox("Skill still on cooldown")?;
										}
										break;
									}
									None => ui.messagebox("Number out of bounds")?,
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
							if let Some(num) = ui.messagebox_with_options(
								"Pick status",
								&manual_statuses_list,
								true,
							)? {
								get_player_mut!(players, id).statuses.drain_by_id(
									*manual_statuses.get(*num).ok_or(anyhow::Error::msg(
										"Couldn't drain manual status",
									))?,
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
		log::debug!("In the character menu...");
		// TODO: create a UI agnostic list state tracker
		let mut state = ListState::default();
		state.next(players.len());
		loop {
			let player_names_list = players
				.iter()
				.map(|(_, pl)| pl.name.as_str())
				.collect::<Vec<&str>>();
			match ui.draw_editor(
				EditorMode::View {
					selected: state.selected_onum(),
				},
				Some("Players"),
				&player_names_list,
				Some(|rect| {
					if let Some(selected) = state.selected_onum() {
						Ui::player_stats(
							players.get_by_index(selected).unwrap().1,
							stat_list,
							status_list,
							rect,
							None,
							None,
							None,
						)
					} else {
						Vec::new()
					}
				}),
			)? {
				EditorAction::View(EditorActionViewMode::Add) => {
					state.select(Some(player_names_list.len()));
					let id = players.push(Player::default());
					log::debug!("Added a new player with #{:?}", id);
					edit_player(ui, players, id, stat_list, status_list)?;
					// TODO: find out which pos the new player has in the list
					//last_selected = Some(id);
				}
				EditorAction::View(EditorActionViewMode::Edit) => {
					if let Some(num) = state.selected_onum() {
						log::debug!("Editing player #{:?}", num);
						edit_player(
							ui,
							players,
							*players.get_by_index(num).unwrap().0,
							stat_list,
							status_list,
						)?;
					}
				}
				EditorAction::View(EditorActionViewMode::Delete) => {
					if let Some(num) = state.selected_onum() {
						log::debug!("Confirming deletion of player #{:?}", num);
						if ui.messagebox_yn("Are you sure?")? {
							log::debug!("Deleting #{:?}", num);
							state.next(player_names_list.len() - 1);
							players.remove(*players.get_by_index(num).unwrap().0);
						} else {
							log::debug!("Not confirmed");
						}
					}
				}
				EditorAction::View(EditorActionViewMode::Next) => {
					state.next(player_names_list.len());
				}
				EditorAction::View(EditorActionViewMode::Prev) => {
					state.prev(player_names_list.len());
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
	}

	fn edit_player(
		ui: &Ui,
		players: &mut Players,
		id: Uid,
		stat_list: &StatList,
		status_list: &StatusList,
	) -> Result<()> {
		log::debug!("Editing player #{}", id);
		let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
		let mut buffer = None;
		let mut error = None;
		loop {
			if buffer.is_none() {
				buffer = try {
					match selected_field {
						PlayerField::Name => players.get(id)?.name.clone(),
						PlayerField::Stat(num) => players
							.get(id)?
							.stats
							.get(stat_list.get(num).unwrap())
							.to_string(),
						PlayerField::SkillName(num) => players
							.get(id)?
							.skills
							.get(*num)
							.map(|x| x.name.clone())
							.unwrap_or_default(),
						PlayerField::SkillCD(num) => players
							.get(id)?
							.skills
							.get(*num)
							.map(|x| x.cooldown.to_string())
							.unwrap_or_default(),
					}
				};
				// if still empty for some reason -> create an empty string
				buffer = Some(buffer.unwrap_or_default());
			}

			// init fields if they don't exist
			match selected_field {
				PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
					let player = get_player_mut!(players, id);
					if player.skills.get(*skill_id).is_none() {
						log::debug!(
							"Going to modify a skill but it doesn't yet exist. Creating..."
						);
						player.skills.push(Skill::default())
					}
				}
				_ => (),
			}

			let player_names_list = players
				.iter()
				.map(|(_, pl)| pl.name.as_str())
				.collect::<Vec<&str>>();

			match ui.draw_editor(
				EditorMode::Edit {
					selected: players.get_index_of(id).unwrap(),
					error: error.clone(),
				},
				Some("Players"),
				&player_names_list,
				Some(|rect| {
					Ui::player_stats(
						players.get(id).unwrap(),
						stat_list,
						status_list,
						rect,
						Some(id),
						Some(selected_field),
						Some(buffer.as_deref().unwrap()),
					)
				}),
			)? {
				EditorAction::Edit(EditorActionEditMode::Char(ch)) => {
					let buffer = buffer.as_mut().unwrap();
					buffer.push(ch);
					if let PlayerField::Stat(_) | PlayerField::SkillCD(_) = selected_field {
						error = if buffer.parse::<i64>().is_err() {
							Some(format!("{} is not a valid number", buffer))
						} else {
							None
						}
					}
				}
				EditorAction::Edit(EditorActionEditMode::Pop) => {
					let buffer = buffer.as_mut().unwrap();
					buffer.pop();
					if let PlayerField::Stat(_) | PlayerField::SkillCD(_) = selected_field {
						error = if buffer.parse::<i64>().is_err() {
							Some(format!("{} is not a valid number", buffer))
						} else {
							None
						}
					}
				}
				EditorAction::Edit(EditorActionEditMode::Next) => {
					selected_field = selected_field.next(stat_list);
					buffer = None;
				}
				EditorAction::Edit(EditorActionEditMode::Prev) => {
					selected_field = selected_field.prev(stat_list);
					buffer = None;
				}
				EditorAction::Edit(EditorActionEditMode::DoneWithField) => {
					let buff_str = buffer.as_mut().unwrap();
					let player = get_player_mut!(players, id);
					match selected_field {
						PlayerField::Name => {
							log::debug!(
								"Editing player #{} name: from {} to {}",
								id,
								player.name,
								buff_str
							);
							if buff_str.is_empty() {
								continue;
							}
							player.name = buff_str.clone();
						}
						PlayerField::Stat(selected) => {
							let stat = stat_list.get(selected).unwrap();

							if let Ok(parsed) = buff_str
								.parse::<i32>()
								//.map_err(|e| log::error!("Error parsing new {:?} value: {}", stat, e))
								.map_err(|e| {
									log::error!("Error parsing new stat {} value: {}", stat, e)
								}) {
								log::debug!(
									//"Chaning player #{} stat {:?}: from {} to {}",
									"Chaning player #{} stat {} to {}",
									id,
									stat,
									parsed
								);
								player.stats.set(stat, parsed);
							} else {
								continue;
							}
						}
						PlayerField::SkillName(skill_id) => {
							let skill_name = &mut player.skills[*skill_id].name;
							log::debug!(
								"Changing player #{}'s skill #{}'s name: from {} to {}",
								id,
								skill_id,
								skill_name,
								buff_str
							);
							*skill_name = buff_str.clone();
						}
						PlayerField::SkillCD(skill_id) => {
							if let Ok(parsed) = buff_str.parse::<u32>().map_err(|e| {
								log::error!("Error parsing new skill #{} CD value: {}", skill_id, e)
							}) {
								let skill_cd = &mut player.skills[*skill_id].cooldown;
								log::debug!(
									"Changing player #{}'s skill #{}'s CD: from {} to {}",
									id,
									skill_id,
									skill_cd,
									parsed
								);
								player.skills[*skill_id].cooldown = parsed;
							}
						}
					}
					buffer = None;
					selected_field = selected_field.next(stat_list);
				}
				// FIXME: properly check for empty buffer in player and skill names
				EditorAction::Edit(EditorActionEditMode::Done) => {
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
				EditorAction::View(_) => {
					log::error!("This should have never been reached. Somehow the editor in editing mode returned a View action");
					unreachable!();
				}
			}
		}

		log::debug!("Exiting out of the character menu...");
		Ok(())
	}

	fn reorder_players(
		ui: &Ui,
		old_player_order: &[Uid],
		players: &mut Players,
	) -> Result<Vec<Uid>> {
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
						let name_list: Vec<&str> =
							player_list.iter().map(|(_, name)| *name).collect();
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
	}

	fn statlist_menu(ui: &Ui, stat_list: &mut StatList) -> Result<()> {
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
						stat_list.insert(ui.edit_setlist(
							stat_list,
							stat.1,
							stat.0,
							Some("Stats"),
						)?);
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
	}

	fn statuslist_menu(ui: &Ui, status_list: &mut StatusList) -> Result<()> {
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
	}
}