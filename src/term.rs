pub mod list_state_ext;

use crate::action_enums::{
	EditorAction, EditorActionEditMode, EditorActionViewMode, GameAction, MainMenuAction,
	SettingsAction,
};
use crate::id::{OrderNum, Uid};
use crate::list::IdList;
use crate::player::{Player, Players};
use crate::player_field::PlayerField;
use crate::skill::Skill;
use crate::stats::StatList;
use crate::status::{Status, StatusCooldownType, StatusList};
use crate::term::list_state_ext::ListStateExt;
use anyhow::Result;
use crossterm::event::{read as read_event, Event, KeyCode};
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::io::{stdout, Stdout};
use tui::{
	backend::CrosstermBackend,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
	Terminal,
};

#[derive(Clone)]
pub enum EditorMode {
	View {
		selected: Option<OrderNum>,
	},
	Edit {
		selected: OrderNum,
		error: Option<String>,
	},
}

enum StatusBarType {
	Normal,
	Error,
}

pub struct Term {
	term: RefCell<Terminal<CrosstermBackend<Stdout>>>,
}

impl Term {
	pub fn new() -> Result<Term> {
		crossterm::terminal::enable_raw_mode()?;
		Ok(Term {
			term: RefCell::new(Terminal::new(CrosstermBackend::new(stdout()))?),
		})
	}

	fn get_window_size(&self, window: Rect) -> (Rect, Rect) {
		let layout = Layout::default()
			.direction(Direction::Vertical)
			.constraints([Constraint::Min(10), Constraint::Length(1)].as_ref())
			.split(window);

		(layout[0], layout[1])
	}

	fn stylize_statusbar<'a, T: Into<Text<'a>>>(text: T, sbtype: StatusBarType) -> Paragraph<'a> {
		let style = match sbtype {
			StatusBarType::Normal => Style::default().bg(Color::Gray).fg(Color::Black),
			StatusBarType::Error => Style::default().bg(Color::Red).fg(Color::White),
		};
		Paragraph::new(text.into()).style(style)
	}

	fn get_centered_box(frame: Rect, width: u16, height: u16) -> Rect {
		let offset_x = (frame.width - width) / 2;
		let offset_y = (frame.height - height) / 2;

		let layout_x = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Length(offset_y),
					Constraint::Length(height),
					Constraint::Length(offset_y),
				]
				.as_ref(),
			)
			.split(frame);

		Layout::default()
			.direction(Direction::Horizontal)
			.constraints(
				[
					Constraint::Length(offset_x),
					Constraint::Length(width),
					Constraint::Length(offset_x),
				]
				.as_ref(),
			)
			.split(layout_x[1])[1]
	}

	fn get_messagebox_text_input_locations(messagebox: Rect) -> (Rect, Rect) {
		let layout_x = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					Constraint::Length(2), // border + space
					Constraint::Length(1), // the text
					Constraint::Length(1), // space
					Constraint::Length(1), // buttons
					Constraint::Length(2), // space + border
				]
				.as_ref(),
			)
			.split(messagebox);

		(
			// the 4 is 2 borders and 2 margins
			Layout::default()
				.direction(Direction::Horizontal)
				.constraints(
					[
						Constraint::Length(2),
						Constraint::Length(messagebox.width - 4),
						Constraint::Length(2),
					]
					.as_ref(),
				)
				.split(layout_x[1])[1],
			Layout::default()
				.direction(Direction::Horizontal)
				.constraints(
					[
						Constraint::Length(2),
						Constraint::Length(messagebox.width - 4),
						Constraint::Length(2),
					]
					.as_ref(),
				)
				.split(layout_x[3])[1],
		)
	}

	pub fn messagebox_with_options_immediate<T: AsRef<str>>(
		&self,
		desc: &str,
		options: &[T],
		selected: Option<OrderNum>,
		is_vertical: bool,
	) -> Result<KeyCode> {
		self.term.borrow_mut().clear()?;
		if options.is_empty() {
			panic!("Can't show a dialog with no buttons")
		}
		let width = {
			let desc_width = desc.len() as u16 + 4;
			let button_width = {
				if !is_vertical {
					// add all button text together
					options
						.iter()
						.map(|item| item.as_ref().chars().count() as u16)
						.sum::<u16>() + 4
				} else {
					// find the longest button text
					options.iter().fold(0, |acc, item| {
						let len = item.as_ref().chars().count();
						if len > acc {
							len
						} else {
							acc
						}
					}) as u16 + 4
				}
			};

			if desc_width > button_width {
				desc_width
			} else {
				button_width
			}
		};
		let height = if !is_vertical {
			7
		} else {
			6 + options.len() as u16
		};

		let mut state = ListState::default();
		state.select_onum(selected);
		loop {
			self.term.borrow_mut().draw(|frame| {
				let block_rect = Term::get_centered_box(frame.size(), width, height);
				let (desc_rect, buttons_rect) =
					Term::get_messagebox_text_input_locations(block_rect);

				let block = Block::default().borders(Borders::ALL);
				let desc = Paragraph::new(desc).alignment(Alignment::Center);
				frame.render_widget(block.clone(), block_rect);
				frame.render_widget(desc, desc_rect);

				if !is_vertical {
					const OFFSET_BETWEEN_BUTTONS: u16 = 3;
					let buttons_rect = {
						let offset = {
							let mut tmp = buttons_rect.width;
							tmp -= options
								.iter()
								.map(|item| item.as_ref().chars().count() as u16)
								.sum::<u16>();
							// if more than out button, substract spacing between them
							if options.len() > 1 {
								tmp -= OFFSET_BETWEEN_BUTTONS * (options.len() as u16 - 1);
							}
							tmp /= 2;

							tmp
						};

						let mut tmp = buttons_rect;
						tmp.x += offset;
						tmp
					};

					for (i, option) in options.iter().enumerate() {
						let button_style = if i == state.selected().unwrap_or(0) {
							Style::default().bg(Color::White).fg(Color::Black)
						} else {
							Style::default()
						};

						let button = Paragraph::new(option.as_ref()).style(button_style);

						let rect = {
							let mut tmp = buttons_rect;
							tmp.width = option.as_ref().chars().count() as u16;
							if i > 0 {
								tmp.x += options[i - 1].as_ref().len() as u16;
								tmp.x += OFFSET_BETWEEN_BUTTONS;
							}

							tmp
						};

						frame.render_widget(button, rect);
					}
				} else {
					for (i, option) in options.iter().enumerate() {
						let rect = {
							let mut tmp = buttons_rect;
							tmp.y += i as u16;
							tmp.width = option.as_ref().chars().count() as u16;
							tmp
						};

						let button_style = if i == state.selected().unwrap_or(0) {
							Style::default().bg(Color::White).fg(Color::Black)
						} else {
							Style::default()
						};

						let button = Paragraph::new(option.as_ref()).style(button_style);
						frame.render_widget(button, rect);
					}
				}
			})?;

			if let Event::Key(key) = read_event()? {
				return Ok(key.code);
			}
		}
	}

	pub fn messagebox_with_options<T: AsRef<str>>(
		&self,
		desc: &str,
		options: &[T],
		is_vertical: bool,
	) -> Result<Option<OrderNum>> {
		let mut state = ListState::default();
		state.select(Some(0));
		loop {
			match self.messagebox_with_options_immediate(
				desc,
				options,
				state.selected_onum(),
				is_vertical,
			)? {
				KeyCode::Enter => return Ok(Some(state.selected_onum().unwrap_or(0.into()))),
				KeyCode::Char(ch) => {
					if let Some(num) = ch.to_digit(10) {
						let num: OrderNum = (num as usize - 1).into();
						if num < options.len().into() {
							return Ok(Some(num));
						}
					}
				}
				KeyCode::Esc => return Ok(None),
				KeyCode::Right if !is_vertical => {
					state.next(options.len());
				}
				KeyCode::Left if !is_vertical => {
					state.prev(options.len());
				}
				KeyCode::Down if is_vertical => {
					state.next(options.len());
				}
				KeyCode::Up if is_vertical => {
					state.prev(options.len());
				}
				_ => (),
			}
		}
	}

	pub fn messagebox_with_input_field(&self, desc: &str) -> Result<String> {
		self.term.borrow_mut().clear()?;
		let width = desc.len() as u16 + 4;
		let height = 7;
		let mut buffer = String::new();

		loop {
			self.term.borrow_mut().draw(|frame| {
				let block_rect = Term::get_centered_box(frame.size(), width, height);
				let (desc_rect, input_rect) = Term::get_messagebox_text_input_locations(block_rect);

				let block = Block::default().borders(Borders::ALL);
				let desc = Paragraph::new(desc).alignment(Alignment::Center);
				let input = Paragraph::new(buffer.as_str());
				frame.render_widget(block.clone(), block_rect);
				frame.render_widget(desc, desc_rect);
				frame.render_widget(input, input_rect);
			})?;

			if let Event::Key(key) = read_event()? {
				match key.code {
					KeyCode::Char(ch) => buffer.push(ch),
					KeyCode::Backspace => {
						buffer.pop();
					}
					KeyCode::Enter => {
						return Ok(buffer);
					}
					_ => (),
				}
			}
		}
	}

	pub fn messagebox_yn(&self, desc: &str) -> Result<bool> {
		Ok(matches!(
			self.messagebox_with_options(desc, &["Yes", "No"], false)?,
			Some(OrderNum(0))
		))
	}

	pub fn messagebox(&self, desc: &str) -> Result<()> {
		self.messagebox_with_options(desc, &["OK"], false)?;
		Ok(())
	}

	pub fn draw_menu<T>(&self, items: &[&str], statusbar_text: T) -> Result<Option<usize>>
	where
		T: AsRef<str>,
	{
		self.term.borrow_mut().clear()?;

		let mut list_state = ListState::default();
		list_state.select(Some(0));
		loop {
			self.term.borrow_mut().draw(|frame| {
				let longest_len = items.iter().fold(0, |acc, item| {
					let len = item.chars().count();
					if len > acc {
						len
					} else {
						acc
					}
				});
				let list = List::new(
					items
						.iter()
						.map(|item| ListItem::new(*item))
						.collect::<Vec<ListItem>>(),
				)
				.highlight_style(Style::default().bg(Color::White).fg(Color::Black));

				let (win_rect, statusbar_rect) = self.get_window_size(frame.size());
				let menu_location = Term::get_centered_box(
					win_rect,
					longest_len as u16 + 4,
					items.len() as u16 + 4,
				);
				frame.render_stateful_widget(list, menu_location, &mut list_state);
				frame.render_widget(
					Term::stylize_statusbar(statusbar_text.as_ref(), StatusBarType::Normal),
					statusbar_rect,
				);
			})?;

			if let Event::Key(key) = read_event()? {
				match key.code {
					KeyCode::Esc => return Ok(None),
					KeyCode::Char(ch) => match ch {
						'0'..='9' => {
							let i = ch.to_digit(10).unwrap() as usize;
							if let Some(id) = i.checked_sub(1) {
								if id < items.len() {
									return Ok(Some(id));
								}
							}
						}
						'q' => return Ok(None),
						_ => (),
					},
					KeyCode::Down => {
						list_state.next(items.len());
					}
					KeyCode::Up => {
						list_state.prev(items.len());
					}
					KeyCode::Enter => {
						if let Some(i) = list_state.selected() {
							assert!(i < items.len());
							return Ok(Some(i));
						}
					}
					_ => (),
				}
			}
		}
	}

	pub fn draw_main_menu(&self) -> Result<MainMenuAction> {
		let items = [
			"Start game",
			"Manage characters",
			"Change player order",
			"Settings",
			"Save and quit",
		];

		let statusbar_text = format!(" dnd-gm-helper v{}", env!("CARGO_PKG_VERSION"));

		loop {
			return Ok(match self.draw_menu(&items, statusbar_text.as_str())? {
				Some(0) => MainMenuAction::Play,
				Some(1) => MainMenuAction::EditPlayers,
				Some(2) => MainMenuAction::ReorderPlayers,
				Some(3) => MainMenuAction::Settings,
				Some(4) | None => {
					if self.messagebox_yn("Are you sure you want to quit?")? {
						MainMenuAction::Quit
					} else {
						continue;
					}
				}
				_ => unreachable!(),
			});
		}
	}

	pub fn draw_settings_menu(&self) -> Result<SettingsAction> {
		let items = ["Edit Stats", "Edit Statuses", "Go back..."];

		let statusbar_text = " Settings";

		Ok(match self.draw_menu(&items, statusbar_text)? {
			Some(0) => SettingsAction::EditStats,
			Some(1) => SettingsAction::EditStatuses,
			Some(2) | None => SettingsAction::GoBack,
			_ => unreachable!(),
		})
	}

	pub fn draw_game(
		&self,
		player: &Player,
		stat_list: &StatList,
		status_list: &StatusList,
	) -> Result<GameAction> {
		loop {
			self.term.borrow_mut().draw(|frame| {
				let (window_rect, statusbar_rect) = self.get_window_size(frame.size());

				let mut player_stats = Term::player_stats(
					player,
					stat_list,
					status_list,
					window_rect,
					None,
					None,
					None,
				);
				while let Some((table, table_rect)) = player_stats.pop() {
					frame.render_widget(table, table_rect);
				}

				let delimiter = Span::raw(" | ");
				let style_underlined = Style::default().add_modifier(Modifier::UNDERLINED);
				let statusbar_text = Spans::from(vec![
					" Use ".into(),
					Span::styled("s", style_underlined),
					"kill".into(),
					delimiter.clone(),
					Span::styled("A", style_underlined),
					"dd status".into(),
					delimiter.clone(),
					Span::styled("D", style_underlined),
					"rain status".into(),
					delimiter.clone(),
					Span::styled("C", style_underlined),
					"lear statuses".into(),
					", ".into(),
					"skill CD :".into(),
					Span::styled("v", style_underlined),
					delimiter.clone(),
					"Manage ".into(),
					Span::styled("m", style_underlined),
					"oney".into(),
					delimiter.clone(),
					"Next turn: \"".into(),
					Span::styled(" ", style_underlined),
					"\"".into(),
					delimiter.clone(),
					"Ski".into(),
					Span::styled("p", style_underlined),
					" turn".into(),
					delimiter.clone(),
					"Pick next pl.: ".into(),
					Span::styled("o", style_underlined),
					delimiter.clone(),
					Span::styled("Q", style_underlined),
					"uit".into(),
				]);

				frame.render_widget(
					Term::stylize_statusbar(statusbar_text, StatusBarType::Normal),
					statusbar_rect,
				);
			})?;

			if let Event::Key(key) = read_event()? {
				match key.code {
					KeyCode::Char(ch) => match ch {
						's' => return Ok(GameAction::UseSkill),
						'a' => return Ok(GameAction::AddStatus),
						'd' => {
							match self.messagebox_with_options(
								"Which statuses to drain?",
								&["On attacking", "On getting attacked", "Manual"],
								true,
							)? {
								Some(OrderNum(0)) => {
									return Ok(GameAction::DrainStatus(
										StatusCooldownType::OnAttacking,
									))
								}
								Some(OrderNum(1)) => {
									return Ok(GameAction::DrainStatus(
										StatusCooldownType::OnGettingAttacked,
									))
								}
								Some(OrderNum(2)) => {
									return Ok(GameAction::DrainStatus(StatusCooldownType::Manual))
								}
								_ => (),
							}
						}
						'c' => return Ok(GameAction::ClearStatuses),
						'v' => return Ok(GameAction::ResetSkillsCD),
						//'m' => return GameAction::ManageMoney,
						'm' => self.messagebox("Turned off for now.")?,
						' ' => return Ok(GameAction::MakeTurn),
						'p' => return Ok(GameAction::SkipTurn),
						'o' => return Ok(GameAction::NextPlayerPick),
						'q' => return Ok(GameAction::Quit),
						_ => (),
					},
					KeyCode::Esc => return Ok(GameAction::Quit),
					_ => (),
				}
			}
		}
	}

	pub fn player_stats<'a>(
		player: &'a Player,
		stat_list: &'a StatList,
		status_list: &StatusList,
		rect: Rect,
		player_id: Option<Uid>,
		selected: Option<PlayerField>,
		selected_str: Option<&'a str>,
	) -> Vec<(Table<'a>, Rect)> {
		let selected_style = Style::default().bg(Color::White).fg(Color::Black);
		let mut rows_outer = Vec::new();

		let id_str = player_id
			.map(|id| id.to_string())
			.unwrap_or_else(|| "".to_string());
		let id_str = if !id_str.is_empty() {
			format!("ID: {}", id_str)
		} else {
			id_str
		};

		rows_outer.push(if let Some(PlayerField::Name) = selected {
			let name = match selected_str {
				Some(string) => string,
				None => player.name.as_str(),
			};
			Row::new::<[Cell; 3]>(["Name".into(), name.into(), id_str.into()]).style(selected_style)
		} else {
			Row::new::<[Cell; 3]>(["Name".into(), player.name.as_str().into(), id_str.into()])
		});

		//rows.push(Row::new(["Stats"]));

		let mut rows_stats = Vec::new();
		{
			for (i, stat) in stat_list.iter().enumerate() {
				// TODO: avoid to_string()'ing everything
				// TODO: make this actually readable and easy to understand
				let (style, stat_text) = match (selected, selected_str) {
					(Some(selected), Some(string)) => {
						if let PlayerField::Stat(selected) = selected {
							if *selected == i {
								(selected_style, string.to_string())
							} else {
								(Style::default(), player.stats.get(stat).to_string())
							}
						} else {
							(Style::default(), player.stats.get(stat).to_string())
						}
					}
					(_, _) => {
						if let Some(PlayerField::Stat(selected)) = selected {
							if *selected == i {
								(selected_style, player.stats.get(stat).to_string())
							} else {
								(Style::default(), player.stats.get(stat).to_string())
							}
						} else {
							(Style::default(), player.stats.get(stat).to_string())
						}
					}
				};
				rows_stats.push(
					Row::new::<[Cell; 2]>([
						//stat_list.get(stat_id).unwrap().to_string().into(),
						stat.as_str().into(),
						stat_text.into(),
					])
					.style(style),
				);
			}
		}

		//rows.push(Row::new(["Skills"]));
		let mut rows_skills = Vec::new();

		for (i, skill) in player.skills.iter().enumerate() {
			// TODO: dedup!!!
			let mut name_style = None;
			let name: String;
			if let Some(PlayerField::SkillName(curr_skill_num)) = selected {
				if *curr_skill_num == i {
					name = if let Some(selected_str) = selected_str {
						selected_str.into()
					} else {
						skill.name.as_str().into()
					};
					name_style = Some(selected_style);
				} else {
					name = skill.name.as_str().into();
				}
			} else {
				name = skill.name.as_str().into();
			}

			let cd_string = skill.cooldown.to_string();
			let mut cd_style = None;
			let cd: String;
			if let Some(PlayerField::SkillCD(curr_skill_num)) = selected {
				if *curr_skill_num == i {
					cd = if let Some(selected_str) = selected_str {
						selected_str.into()
					} else {
						cd_string
					};
					cd_style = Some(selected_style);
				} else {
					cd = cd_string;
				}
			} else {
				cd = cd_string;
			};

			rows_skills.push(Row::new::<[Cell; 2]>([
				Span::styled(name, name_style.unwrap_or_default()).into(),
				Span::styled(
					format!("CD: {} of {}", skill.cooldown_left.to_string(), cd),
					cd_style.unwrap_or_default(),
				)
				.into(),
			]));
		}

		let mut rows_statuses = Vec::new();

		for (_, status) in player.statuses.iter() {
			rows_statuses.push(Row::new::<[Cell; 2]>([
				status.status_type.as_str().into(),
				format!(
					"{} turns left ({:?})",
					status.duration_left, status.status_cooldown_type
				)
				.into(),
			]));
		}

		/*
		rows.push(
			Row::new::<Vec<Cell>>(vec!["Money".into(), player.money.to_string().into()]).style(
				if let Some(PlayerField::Money) = selected {
					selected_style.clone()
				} else {
					Style::default()
				},
			),
		);
		*/

		let rows_statuses_len = rows_statuses.len();
		let layout = Layout::default()
			.direction(Direction::Vertical)
			.constraints(
				[
					// TODO: replace as with try_into()
					Constraint::Length(rows_outer.len() as u16),
					Constraint::Length(rows_stats.len() as u16 + 2), // + borders
					Constraint::Length(rows_skills.len() as u16 + 2),
					Constraint::Length(if rows_statuses_len > 0 {
						rows_statuses_len as u16 + 2
					} else {
						0
					}),
					Constraint::Min(1),
				]
				.as_ref(),
			)
			.split(rect);

		let table_outer = Table::new(rows_outer).widths(
			[
				Constraint::Length(10),
				Constraint::Length(20),
				Constraint::Min(5),
			]
			.as_ref(),
		);

		let table_stats = Table::new(rows_stats)
			.widths([Constraint::Length(15), Constraint::Min(5)].as_ref())
			.block(Block::default().borders(Borders::ALL).title("Stats"));

		let table_skills = Table::new(rows_skills)
			.widths([Constraint::Length(30), Constraint::Length(30)].as_ref())
			.block(Block::default().borders(Borders::ALL).title("Skills"));

		let table_statuses = Table::new(rows_statuses)
			.widths([Constraint::Length(30), Constraint::Length(30)].as_ref())
			.block(Block::default().borders(Borders::ALL).title("Statuses"));

		let [rect_outer, rect_stats, rect_skills, rect_statuses, _] =
			<[Rect; 5]>::try_from(layout).ok().unwrap();

		let mut stats = vec![
			(table_outer, rect_outer),
			(table_stats, rect_stats),
			(table_skills, rect_skills),
		];

		if rows_statuses_len > 0 {
			stats.push((table_statuses, rect_statuses));
		}

		stats
	}

	pub fn choose_skill(&self, skills: &[Skill]) -> Result<Option<OrderNum>> {
		self.messagebox_with_options(
			"Select skill",
			skills
				.iter()
				.map(|skill| skill.name.as_str())
				.collect::<Vec<&str>>()
				.as_slice(),
			true,
		)
	}

	pub fn choose_status(&self, status_list: &StatusList) -> Result<Option<Status>> {
		let status_type = match self.messagebox_with_options(
			"Choose a status",
			&status_list.get_names(),
			true,
		)? {
			Some(num) => status_list.get(num).unwrap(),
			None => return Ok(None),
		};

		let status_cooldown_type = match self.messagebox_with_options(
			"Status cooldown type",
			&["Normal", "On attacking", "On getting attacked", "Manual"],
			true,
		)? {
			Some(num) => match *num {
				0 => StatusCooldownType::Normal,
				1 => StatusCooldownType::OnAttacking,
				2 => StatusCooldownType::OnGettingAttacked,
				3 => StatusCooldownType::Manual,
				_ => unreachable!(),
			},
			None => return Ok(None),
		};

		let duration_left = loop {
			match self
				.messagebox_with_input_field("Status duration")?
				.parse::<u32>()
			{
				Ok(num) => break num,
				Err(_) => self.messagebox("Not a valid number")?,
			}
		};

		Ok(Some(Status::new(
			status_type.to_string(),
			status_cooldown_type,
			duration_left,
		)))
	}

	pub fn get_money_amount(&self) -> Result<i64> {
		loop {
			let input = self.messagebox_with_input_field("Add or remove money")?;

			let input: i64 = match input.parse() {
				Ok(num) => num,
				Err(_) => {
					self.messagebox(
						format!("{} is not a valid input. Good examples: 500, -68", input).as_str(),
					)?;
					continue;
				}
			};

			return Ok(input);
		}
	}

	pub fn pick_player<'a>(&self, players: &'a mut Players) -> Result<Option<&'a Player>> {
		let player_list = players
			.iter()
			.map(|(_, x)| x.name.as_str())
			.collect::<Vec<&str>>();
		return Ok(
			match self.messagebox_with_options("Pick a player", &player_list, true)? {
				Some(num) => Some(players.get_by_index(num).unwrap().1),
				None => None,
			},
		);
	}

	// TODO: implement "simple editor" that will act as an editor for both stats and statuses (and
	// maybe items???)

	pub fn draw_editor<'a, T, TT, F>(
		&self,
		mode: EditorMode,
		list_title: T,
		list_items: &'a [TT],
		details: Option<F>,
	) -> Result<EditorAction>
	where
		T: AsRef<str>,
		// TODO: why 2 bounds?
		TT: AsRef<str>,
		// TODO: F: Fn(Rect) -> Vec<(Box<dyn Widget>, Rect)>,
		F: Fn(Rect) -> Vec<(Table<'a>, Rect)>,
	{
		// TODO: mv avoid allocating a whole vec?
		let list = List::new({
			let mut v = Vec::new();
			for item in list_items {
				v.push(ListItem::new(item.as_ref()));
			}
			v
		})
		.highlight_symbol(">> ")
		.block(
			Block::default()
				.title(list_title.as_ref())
				.borders(Borders::ALL),
		);
		let mut list_state = ListState::default();
		list_state.select_onum(match mode {
			EditorMode::View { selected } => selected,
			EditorMode::Edit { selected, .. } => Some(selected),
		});

		static STYLE_UNDERLINED: Lazy<Style> =
			Lazy::new(|| Style::default().add_modifier(Modifier::UNDERLINED));
		static DELIMITER: Lazy<Span> = Lazy::new(|| Span::raw(" | "));
		// rendering loop
		loop {
			self.term.borrow_mut().draw(|frame| {
				let (content_rect, statusbar_rect) = self.get_window_size(frame.size());

				// statusbar
				let editor_mode_no_errors = if let EditorMode::Edit { error, .. } = &mode {
					error.is_none()
				} else {
					true
				};
				if editor_mode_no_errors {
					let statusbar_text = match &mode {
						EditorMode::View { selected: _ } => Spans::from(vec![
							" ".into(),
							Span::styled("A", *STYLE_UNDERLINED),
							"dd".into(),
							DELIMITER.clone(),
							Span::styled("E", *STYLE_UNDERLINED),
							"dit".into(),
							DELIMITER.clone(),
							Span::styled("D", *STYLE_UNDERLINED),
							"elete".into(),
							DELIMITER.clone(),
							Span::styled("Q", *STYLE_UNDERLINED),
							"uit".into(),
						]),
						EditorMode::Edit { .. } => Spans::from(" Edit mode. Press ESC to quit"),
					};

					frame.render_widget(
						Term::stylize_statusbar(statusbar_text, StatusBarType::Normal),
						statusbar_rect,
					);
				} else {
					// always true here
					if let EditorMode::Edit { error, .. } = &mode {
						frame.render_widget(
							Term::stylize_statusbar(
								error.as_deref().unwrap(),
								StatusBarType::Error,
							),
							statusbar_rect,
						);
					}
				}

				let [list_rect, details_rect] = {
					if details.is_some() {
						let tables = Layout::default()
							.direction(Direction::Horizontal)
							.constraints(
								[Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
							)
							.split(content_rect);

						<[Rect; 2]>::try_from(tables).ok().unwrap()
					} else {
						[content_rect, Rect::default()]
					}
				};

				frame.render_stateful_widget(list.clone(), list_rect, &mut list_state);

				if let Some(details) = details.as_ref() {
					let mut widgets = details(details_rect);
					while let Some((widget, widget_rect)) = widgets.pop() {
						frame.render_widget(widget, widget_rect);
					}
				}
			})?;

			if let Event::Key(key) = read_event()? {
				match mode {
					EditorMode::View { .. } => match key.code {
						KeyCode::Char(ch) => match ch {
							'a' => return Ok(EditorAction::View(EditorActionViewMode::Add)),
							'e' => return Ok(EditorAction::View(EditorActionViewMode::Edit)),
							'd' => return Ok(EditorAction::View(EditorActionViewMode::Delete)),
							'q' => return Ok(EditorAction::View(EditorActionViewMode::Quit)),
							_ => (),
						},
						KeyCode::Down => return Ok(EditorAction::View(EditorActionViewMode::Next)),

						KeyCode::Up => return Ok(EditorAction::View(EditorActionViewMode::Prev)),

						KeyCode::Esc => return Ok(EditorAction::View(EditorActionViewMode::Quit)),
						_ => (),
					},
					EditorMode::Edit { .. } => {
						/*
						macro_rules! validate {
							() => {
								if !validate_input(
									add_mode_buffer.as_ref().unwrap(),
									selected_field,
								) {
									errors.push(format!(
										"Not a valid number: {}",
										add_mode_buffer.as_ref().unwrap()
									));
									false
								} else {
									true
								}
							};
						}
						*/

						match key.code {
							KeyCode::Char(ch) => {
								//buffer.push(ch);
								//validate!();
								return Ok(EditorAction::Edit(EditorActionEditMode::Char(ch)));
							}
							KeyCode::Up => {
								return Ok(EditorAction::Edit(EditorActionEditMode::Prev));
							}
							KeyCode::Down => {
								return Ok(EditorAction::Edit(EditorActionEditMode::Next));
							}
							KeyCode::Backspace => {
								return Ok(EditorAction::Edit(EditorActionEditMode::Pop));
							}
							KeyCode::Enter => {
								return Ok(EditorAction::Edit(EditorActionEditMode::DoneWithField));
							}
							KeyCode::Esc => {
								return Ok(EditorAction::Edit(EditorActionEditMode::Done));
							}
							_ => (),
						}
					}
				}
			}
		}
	}
}
