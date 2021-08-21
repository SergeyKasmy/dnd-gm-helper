pub mod list_state_ext;

use crate::ui::Ui;
use dnd_gm_helper::action_enums::{
	EditorAction, EditorActionEditMode, EditorActionViewMode, GameAction, MainMenuAction,
	SettingsAction,
};
use dnd_gm_helper::id::{OrderNum, Uid};
use dnd_gm_helper::list::SetList;
use dnd_gm_helper::player::{Player, Players};
use dnd_gm_helper::player_field::PlayerField;
use dnd_gm_helper::side_effect::{SideEffect, SideEffectAffects, SideEffectType};
use dnd_gm_helper::skill::Skill;
use dnd_gm_helper::stats::StatList;
use dnd_gm_helper::status::{Status, StatusCooldownType, StatusList};
use list_state_ext::ListStateExt;

use anyhow::Result;
use crossterm::event::{read as read_event, Event, KeyCode};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::io::{stdout, Stdout};
use tui::widgets::Widget;
use tui::{
	backend::CrosstermBackend,
	layout::{Alignment, Constraint, Direction, Layout, Rect},
	style::{Color, Modifier, Style},
	text::{Span, Spans, Text},
	widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
	Terminal,
};

static STYLE_SELECTED: Lazy<Style> =
	Lazy::new(|| Style::default().bg(Color::White).fg(Color::Black));

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

	fn messagebox_custom<F>(
		&self,
		width: u16,
		height: u16,
		desc: impl AsRef<str>,
		contents: F,
		// after which widget to place the buffer and with which offset + the buffer itself
		mut with_buffer: Option<((OrderNum, u16), &mut String)>,
	) -> Result<KeyCode>
	where
		// takes the rect of the window and returns the widgets and their coords
		F: Fn(Rect) -> Vec<(Box<dyn Widget>, Rect)>,
	{
		let desc = desc.as_ref();
		self.term.borrow_mut().clear()?;
		loop {
			self.term.borrow_mut().draw(|frame| {
				let messagebox_rect = Term::get_centered_box(frame.size(), width + 4, height + 4); // +4 cause of the the margins
				let block = Block::default().borders(Borders::ALL).title(desc);
				let inner_rect = {
					let without_margin = block.inner(messagebox_rect);
					let with_margin = Block::default().borders(Borders::ALL);
					with_margin.inner(without_margin)
				};
				frame.render_widget(block, messagebox_rect);

				let mut widgets = contents(inner_rect);
				for (widget, rect) in widgets.iter_mut() {
					frame.render_widget_ref(widget.as_mut(), *rect);
				}

				if let Some(((widget_id, offset), ref buffer)) = with_buffer {
					frame.render_widget(
						Paragraph::new(Span::styled(buffer.as_str(), *STYLE_SELECTED)),
						{
							let mut without_offset = widgets[*widget_id].1;
							without_offset.x += offset;
							without_offset
						},
					)
				}
			})?;

			if let Event::Key(key) = read_event()? {
				match key.code {
					KeyCode::Backspace => {
						if let Some((_, ref mut buffer)) = with_buffer {
							buffer.pop();
						}
					}
					KeyCode::Char(ch) => {
						if let Some((_, ref mut buffer)) = with_buffer {
							buffer.push(ch);
						}
					}
					_ => return Ok(key.code),
				}
			}
		}
	}

	fn messagebox_with_options_immediate(
		&self,
		desc: impl AsRef<str>,
		options: &[impl AsRef<str>],
		selected: Option<OrderNum>,
		is_vertical: bool,
	) -> Result<KeyCode> {
		let desc = desc.as_ref();
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

	fn player_stats<'a>(
		player: &'a Player,
		stat_list: &'a StatList,
		rect: Rect,
		player_id: Option<Uid>,
		selected: Option<PlayerField>,
		selected_str: Option<&'a str>,
	) -> Vec<(Table<'a>, Rect)> {
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
			Row::new::<[Cell; 3]>(["Name".into(), name.into(), id_str.into()])
				.style(*STYLE_SELECTED)
		} else {
			Row::new::<[Cell; 3]>(["Name".into(), player.name.as_str().into(), id_str.into()])
		});

		//rows.push(Row::new(["Stats"]));

		let mut rows_stats = Vec::new();
		{
			for (i, stat) in stat_list.iter().enumerate() {
				// FIXME: avoid to_string()'ing everything
				// FIXME: make this actually readable and easy to understand
				let (style, stat_text) = match (selected, selected_str) {
					(Some(selected), Some(string)) => {
						if let PlayerField::Stat(selected) = selected {
							if *selected == i {
								(*STYLE_SELECTED, string.to_string())
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
								(*STYLE_SELECTED, player.stats.get(stat).to_string())
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
					name_style = Some(*STYLE_SELECTED);
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
					cd_style = Some(*STYLE_SELECTED);
				} else {
					cd = cd_string;
				}
			} else {
				cd = cd_string;
			};

			let mut sideeffect_style = None;
			if let Some(PlayerField::SkillSideEffect(curr_skill_num)) = selected {
				if *curr_skill_num == i {
					sideeffect_style = Some(*STYLE_SELECTED);
				}
			}

			rows_skills.push(Row::new::<[Cell; 3]>([
				Span::styled(name, name_style.unwrap_or_default()).into(),
				Span::styled(
					format!("{} of {}", skill.cooldown_left.to_string(), cd),
					cd_style.unwrap_or_default(),
				)
				.into(),
				Span::styled(
					match &skill.side_effect {
						Some(se) => se.to_string(),
						None => "None".to_string(),
					},
					sideeffect_style.unwrap_or_default(),
				)
				.into(),
			]));
		}
		if !rows_skills.is_empty() {
			//rows_skills.insert(0, Row::new::<[Cell; 3]>([Span::raw("Name").into(), Span::raw("CD").into(), Span::raw("Side Effect").into()]));
			rows_skills.insert(
				0,
				Row::new::<[Cell; 3]>(["Name".into(), "CD".into(), "Side Effect".into()]),
			);
			rows_skills.insert(
				1,
				// NOTE: workaround, just adding an empty row doesn't work for some reason, it
				// shows up last in the table no matter what
				Row::new::<[Cell; 3]>(["".into(), "".into(), "".into()]),
			);
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
			.widths(
				[
					Constraint::Length(30),
					Constraint::Length(30),
					Constraint::Length(30),
				]
				.as_ref(),
			)
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

	fn draw_editor<'a, F>(
		&self,
		mode: EditorMode,
		list_title: Option<impl AsRef<str>>,
		list_items: &[impl AsRef<str>],
		details: Option<F>,
	) -> Result<EditorAction>
	where
		F: Fn(Rect) -> Vec<(Table<'a>, Rect)>,
	{
		let block = {
			let block = Block::default().borders(Borders::ALL);
			if let Some(title) = list_title {
				// FIXME: avoid to_string'ing
				block.title(title.as_ref().to_string())
			} else {
				block
			}
		};
		let list = List::new({
			let mut v = Vec::with_capacity(list_items.len());
			for item in list_items {
				v.push(ListItem::new(item.as_ref()));
			}
			v
		})
		.highlight_symbol(">> ")
		.block(block);
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
							'e' if list_state.selected().is_some() => {
								return Ok(EditorAction::View(EditorActionViewMode::Edit(
									list_state.selected_onum().unwrap(),
								)))
							}
							'd' if list_state.selected().is_some() => {
								return Ok(EditorAction::View(EditorActionViewMode::Delete(
									list_state.selected_onum().unwrap(),
								)))
							}
							'q' => return Ok(EditorAction::View(EditorActionViewMode::Quit)),
							_ => (),
						},
						KeyCode::Down => return Ok(EditorAction::View(EditorActionViewMode::Next)),

						KeyCode::Up => return Ok(EditorAction::View(EditorActionViewMode::Prev)),

						KeyCode::Esc => return Ok(EditorAction::View(EditorActionViewMode::Quit)),
						_ => (),
					},
					EditorMode::Edit { .. } => match key.code {
						KeyCode::Char(ch) => {
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
					},
				}
			}
		}
	}
}

impl Ui for Term {
	fn draw_menu(
		&self,
		items: &[impl AsRef<str>],
		statusbar_text: impl AsRef<str>,
	) -> Result<Option<usize>> {
		self.term.borrow_mut().clear()?;

		let mut list_state = ListState::default();
		list_state.select(Some(0));
		loop {
			self.term.borrow_mut().draw(|frame| {
				let longest_len = items.iter().fold(0, |acc, item| {
					let len = item.as_ref().chars().count();
					if len > acc {
						len
					} else {
						acc
					}
				});
				let list = List::new(
					items
						.iter()
						.map(|item| ListItem::new(item.as_ref()))
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

	fn draw_main_menu(&self) -> Result<MainMenuAction> {
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

	fn draw_settings_menu(&self) -> Result<SettingsAction> {
		let items = ["Edit Stats", "Edit Statuses", "Go back..."];

		let statusbar_text = " Settings";

		Ok(match self.draw_menu(&items, statusbar_text)? {
			Some(0) => SettingsAction::EditStats,
			Some(1) => SettingsAction::EditStatuses,
			Some(2) | None => SettingsAction::GoBack,
			_ => unreachable!(),
		})
	}

	fn draw_game(&self, player: &Player, stat_list: &StatList) -> Result<GameAction> {
		loop {
			self.term.borrow_mut().draw(|frame| {
				let (window_rect, statusbar_rect) = self.get_window_size(frame.size());

				let mut player_stats =
					Term::player_stats(player, stat_list, window_rect, None, None, None);
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

	fn choose_skill(&self, skills: &[Skill]) -> Result<Option<OrderNum>> {
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

	fn choose_status(&self, status_list: &StatusList) -> Result<Option<Status>> {
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

	fn get_money_amount(&self) -> Result<i64> {
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

	// TODO: return the Uid instead
	fn pick_player<'a>(
		&self,
		players: &'a Players,
		ignore: Option<Uid>,
	) -> Result<Option<&'a Player>> {
		let player_list = players
			.iter()
			.filter_map(|(id, x)| {
				if let Some(id_to_ignore) = ignore {
					if id_to_ignore == *id {
						None
					} else {
						Some(x.name.as_str())
					}
				} else {
					Some(x.name.as_str())
				}
			})
			.collect::<Vec<&str>>();
		return Ok(
			match self.messagebox_with_options("Pick a player", &player_list, true)? {
				Some(num) => {
					let chosen_player = players.get_by_index(num).unwrap();
					if let Some(id_to_ignore) = ignore {
						if *chosen_player.0 == id_to_ignore {
							Some(players.get_by_index(OrderNum(*num + 1)).unwrap().1)
						} else {
							Some(chosen_player.1)
						}
					} else {
						Some(chosen_player.1)
					}
				}
				None => None,
			},
		);
	}

	fn draw_character_menu(
		&self,
		players: &Players,
		stat_list: &StatList,
	) -> Result<EditorActionViewMode> {
		log::debug!("In the character menu...");
		// TODO: create a UI agnostic list state tracker
		let mut state = ListState::default();
		state.next(players.len());
		loop {
			let player_names_list = players
				.iter()
				.map(|(_, pl)| pl.name.as_str())
				.collect::<Vec<&str>>();
			match self.draw_editor(
				EditorMode::View {
					selected: state.selected_onum(),
				},
				Some("Players"),
				&player_names_list,
				Some(|rect| {
					if let Some(selected) = state.selected_onum() {
						Term::player_stats(
							players.get_by_index(selected).unwrap().1,
							stat_list,
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
				EditorAction::View(EditorActionViewMode::Next) => {
					state.next(player_names_list.len());
				}
				EditorAction::View(EditorActionViewMode::Prev) => {
					state.prev(player_names_list.len());
				}
				EditorAction::View(action) => return Ok(action),
				_ => unreachable!(),
			}
		}
	}

	fn draw_setlist(&self, setlist: &SetList<String>) -> Result<EditorActionViewMode> {
		log::debug!("In the statlist menu...");
		// TODO: create a UI agnostic list state tracker
		// TODO: preselect the first
		let mut state = ListState::default();
		state.next(setlist.len());
		loop {
			match self.draw_editor(
				EditorMode::View {
					selected: state.selected_onum(),
				},
				Some("Stats"),
				&setlist.get_names(),
				None::<fn(_) -> _>,
			)? {
				EditorAction::View(EditorActionViewMode::Next) => {
					state.next(setlist.len());
				}
				EditorAction::View(EditorActionViewMode::Prev) => {
					state.prev(setlist.len());
				}
				EditorAction::View(action) => return Ok(action),
				EditorAction::Edit(_) => {
					log::error!("How did we even get here??? EditorAction::Edit was somehow returned from the editor not in editing mode. Something went terribly wrong...");
					unreachable!();
				}
			}
		}
	}

	fn edit_player(
		&self,
		players: &Players,
		id: Uid,
		stat_list: &StatList,
		status_list: &StatusList,
	) -> Result<Option<Player>> {
		log::debug!("Editing player #{}", id);
		let mut player_to_edit = players.get(id).unwrap().clone();
		let mut selected_field = PlayerField::Name; // TODO: maybe use something like new()?
		let mut buffer = None;
		let mut error = None;
		loop {
			if buffer.is_none() {
				buffer = match selected_field {
					PlayerField::Name => Some(players.get(id).unwrap().name.clone()),
					PlayerField::Stat(num) => Some(
						player_to_edit
							.stats
							.get(stat_list.get(num).unwrap())
							.to_string(),
					),
					PlayerField::SkillName(num) => Some(
						player_to_edit
							.skills
							.get(*num)
							.map(|x| x.name.clone())
							.unwrap_or_default(),
					),
					PlayerField::SkillCD(num) => Some(
						player_to_edit
							.skills
							.get(*num)
							.map(|x| x.cooldown.to_string())
							.unwrap_or_default(),
					),
					PlayerField::SkillSideEffect(_) => None,
				};
			}

			// init fields if they don't exist
			match selected_field {
				PlayerField::SkillName(skill_id) | PlayerField::SkillCD(skill_id) => {
					if player_to_edit.skills.get(*skill_id).is_none() {
						log::debug!(
							"Going to modify a skill but it doesn't yet exist. Creating..."
						);
						player_to_edit.skills.push(Skill::default())
					}
				}
				_ => (),
			}

			let player_names_list = players
				.iter()
				.map(|(_, pl)| pl.name.as_str())
				.collect::<Vec<&str>>();

			match self.draw_editor(
				EditorMode::Edit {
					selected: players.get_index_of(id).unwrap(),
					error: error.clone(),
				},
				Some("Players"),
				&player_names_list,
				Some(|rect| {
					Term::player_stats(
						&player_to_edit,
						stat_list,
						rect,
						Some(id),
						Some(selected_field),
						buffer.as_deref(),
					)
				}),
			)? {
				EditorAction::Edit(EditorActionEditMode::Char(ch)) => {
					if let PlayerField::SkillSideEffect(_) = selected_field {
						continue;
					}
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
					if let PlayerField::SkillSideEffect(_) = selected_field {
						continue;
					}
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
					match selected_field {
						PlayerField::Name => {
							let buff_str = buffer.as_mut().unwrap();
							log::debug!(
								"Editing player #{} name: from {} to {}",
								id,
								player_to_edit.name,
								buff_str
							);
							if buff_str.is_empty() {
								continue;
							}
							player_to_edit.name = buff_str.clone();
							selected_field = selected_field.next(stat_list);
						}
						PlayerField::Stat(selected) => {
							let buff_str = buffer.as_mut().unwrap();
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
								player_to_edit.stats.set(stat, parsed);
							} else {
								continue;
							}
							selected_field = selected_field.next(stat_list);
						}
						PlayerField::SkillName(skill_id) => {
							let buff_str = buffer.as_mut().unwrap();
							let skill_name = &mut player_to_edit.skills[*skill_id].name;
							log::debug!(
								"Changing player #{}'s skill #{}'s name: from {} to {}",
								id,
								skill_id,
								skill_name,
								buff_str
							);
							*skill_name = buff_str.clone();
							selected_field = selected_field.next(stat_list);
						}
						PlayerField::SkillCD(skill_id) => {
							let buff_str = buffer.as_mut().unwrap();
							if let Ok(parsed) = buff_str.parse::<u32>().map_err(|e| {
								log::error!("Error parsing new skill #{} CD value: {}", skill_id, e)
							}) {
								let skill_cd = &mut player_to_edit.skills[*skill_id].cooldown;
								log::debug!(
									"Changing player #{}'s skill #{}'s CD: from {} to {}",
									id,
									skill_id,
									skill_cd,
									parsed
								);
								player_to_edit.skills[*skill_id].cooldown = parsed;
							}
							selected_field = selected_field.next(stat_list);
						}
						PlayerField::SkillSideEffect(skill_num) => {
							let old_side_effect =
								player_to_edit.skills[*skill_num].side_effect.take();
							log::trace!("Old side effect: {:?}", old_side_effect);
							let new_side_effect =
								self.edit_side_effect(old_side_effect, status_list)?;
							log::trace!("New side effect: {:?}", new_side_effect);
							player_to_edit.skills[*skill_num].side_effect = new_side_effect;
						}
					}
					buffer = None;
				}
				// FIXME: properly check for empty buffer in player and skill names
				EditorAction::Edit(EditorActionEditMode::Done) => {
					log::debug!("Done editing {}", player_to_edit.name);
					if let Some(skill) = player_to_edit.skills.last() {
						if skill.name.is_empty() {
							log::debug!("Last skill's name is empty. Removing...");
							player_to_edit.skills.pop();
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
		Ok(Some(player_to_edit))
	}

	fn edit_setlist(
		&self,
		list: &SetList<String>,
		item: String,
		item_ordernum: OrderNum,
		title: Option<impl AsRef<str>>,
	) -> Result<String> {
		let mut buffer = item;
		loop {
			let item_names = {
				let mut item_names = list.get_names();
				item_names.insert(*item_ordernum, &buffer);
				item_names
			};
			match self.draw_editor(
				EditorMode::Edit {
					selected: item_ordernum,
					error: None,
				},
				title.as_ref(),
				&item_names,
				None::<fn(_) -> _>,
			)? {
				EditorAction::Edit(EditorActionEditMode::Char(ch)) => {
					buffer.push(ch);
				}
				EditorAction::Edit(EditorActionEditMode::Pop) => {
					buffer.pop();
				}
				EditorAction::Edit(
					EditorActionEditMode::DoneWithField
					| EditorActionEditMode::Done
					| EditorActionEditMode::Next
					| EditorActionEditMode::Prev,
				) => {
					break;
				}
				EditorAction::View(_) => {
					log::error!("This should have never been reached. Somehow the editor in editing mode returned a View action");
					unreachable!();
				}
			}
		}

		log::debug!("Exiting out of the setlist editor...");
		Ok(buffer)
	}

	fn edit_side_effect(
		&self,
		old_side_effect: Option<SideEffect>,
		status_list: &StatusList,
	) -> Result<Option<SideEffect>> {
		enum SideEffectField {
			Description,
			Type,
			Affects,
			Remove,
			Done,
		}

		let rect_offset = |rect: &Rect, offset: u16| {
			let mut new_rect = rect.clone();
			new_rect.y += offset;
			new_rect
		};

		let mut selected_field = SideEffectField::Description;
		let (mut desc_buffer, mut r#type, mut affects) =
			if let Some(old_side_effect) = old_side_effect.clone() {
				(
					old_side_effect.description,
					Some(old_side_effect.r#type),
					Some(old_side_effect.affects),
				)
			} else {
				(String::new(), None, None)
			};
		loop {
			// TODO: avoid cloning
			let desc_buffer_clone = desc_buffer.clone();
			match self.messagebox_custom(
				40,
				6,
				"Side effect",
				|rect| {
					let mut widgets: Vec<(Box<dyn Widget>, Rect)> = Vec::new();

					widgets.push((
						Box::new(Paragraph::new(Span::styled(
							if let SideEffectField::Description = selected_field {
								"Description: ".to_string()
							} else {
								format!("Description: {}", desc_buffer_clone)
							},
							if let SideEffectField::Description = selected_field {
								*STYLE_SELECTED
							} else {
								Style::default()
							},
						))),
						rect.clone(),
					));
					widgets.push((
						Box::new(Paragraph::new(Span::styled(
							format!(
								"Type: {}",
								if let Some(r#type) = r#type.as_ref() {
									r#type.to_string()
								} else {
									"None".to_string()
								}
							),
							if let SideEffectField::Type = selected_field {
								*STYLE_SELECTED
							} else {
								Style::default()
							},
						))),
						rect_offset(&rect, 1),
					));
					widgets.push((
						Box::new(Paragraph::new(Span::styled(
							format!(
								"Affects: {}",
								if let Some(affects) = affects.as_ref() {
									affects.to_string()
								} else {
									"None".to_string()
								}
							),
							if let SideEffectField::Affects = selected_field {
								*STYLE_SELECTED
							} else {
								Style::default()
							},
						))),
						rect_offset(&rect, 2),
					));
					widgets.push((
						Box::new(Paragraph::new(Span::styled(
							"Remove",
							if let SideEffectField::Remove = selected_field {
								*STYLE_SELECTED
							} else {
								Style::default()
							},
						))),
						rect_offset(&rect, 4),
					));
					widgets.push((
						Box::new(Paragraph::new(Span::styled(
							"Done",
							if let SideEffectField::Done = selected_field {
								*STYLE_SELECTED
							} else {
								Style::default()
							},
						))),
						rect_offset(&rect, 5),
					));
					widgets
				},
				if let SideEffectField::Description = selected_field {
					Some(((0.into(), 13), &mut desc_buffer))
				} else {
					None
				},
			)? {
				KeyCode::Enter => match selected_field {
					SideEffectField::Description => selected_field = SideEffectField::Type,
					SideEffectField::Type => {
						r#type = Some(
							match self.messagebox_with_options(
								"Side effect type",
								&["Adds status", "Uses skill"],
								true,
							)? {
								Some(OrderNum(0)) => {
									let status = loop {
										if let Some(status) = self.choose_status(status_list)? {
											break status;
										}
									};
									SideEffectType::AddsStatus(status)
								}
								Some(OrderNum(1)) => SideEffectType::UsesSkill,
								None => continue,
								_ => unreachable!(),
							},
						);
						selected_field = SideEffectField::Affects;
					}
					SideEffectField::Affects => {
						affects = Some(
							match self.messagebox_with_options(
								"Affects",
								&["Self", "Someone else", "Both"],
								true,
							)? {
								Some(OrderNum(0)) => SideEffectAffects::Themselves,
								Some(OrderNum(1)) => SideEffectAffects::SomeoneElse,
								Some(OrderNum(2)) => SideEffectAffects::Both,
								None => continue,
								_ => unreachable!(),
							},
						);

						selected_field = SideEffectField::Done;
					}
					SideEffectField::Remove => return Ok(None),
					SideEffectField::Done => {
						if r#type.is_some() && affects.is_some() {
							break;
						}
					}
				},
				KeyCode::Up => {
					selected_field = match selected_field {
						SideEffectField::Description => SideEffectField::Done,
						SideEffectField::Type => SideEffectField::Description,
						SideEffectField::Affects => SideEffectField::Type,
						SideEffectField::Remove => SideEffectField::Affects,
						SideEffectField::Done => SideEffectField::Remove,
					}
				}
				KeyCode::Down => {
					selected_field = match selected_field {
						SideEffectField::Description => SideEffectField::Type,
						SideEffectField::Type => SideEffectField::Affects,
						SideEffectField::Affects => SideEffectField::Remove,
						SideEffectField::Remove => SideEffectField::Done,
						SideEffectField::Done => SideEffectField::Description,
					}
				}
				KeyCode::Esc => return Ok(old_side_effect),
				_ => (),
			}
		}

		Ok(Some(SideEffect {
			description: desc_buffer,
			r#type: r#type.unwrap(),
			affects: affects.unwrap(),
		}))
	}

	fn reorder_players(&self, old_player_order: &[Uid], players: &mut Players) -> Result<Vec<Uid>> {
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
			match self.messagebox_with_options("Choose which player to move", &options, true)? {
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
						match self.messagebox_with_options_immediate(
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

	fn messagebox_with_options(
		&self,
		desc: impl AsRef<str>,
		options: &[impl AsRef<str>],
		is_vertical: bool,
	) -> Result<Option<OrderNum>> {
		let desc = desc.as_ref();
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

	fn messagebox_with_input_field(&self, desc: impl AsRef<str>) -> Result<String> {
		let desc = desc.as_ref();
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

	fn messagebox_yn(&self, desc: impl AsRef<str>) -> Result<bool> {
		Ok(matches!(
			self.messagebox_with_options(desc, &["Yes", "No"], false)?,
			Some(OrderNum(0))
		))
	}

	fn messagebox(&self, desc: impl AsRef<str>) -> Result<()> {
		self.messagebox_with_options(desc.as_ref(), &["OK"], false)?;
		Ok(())
	}
}
