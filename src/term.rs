use crate::{Player, Players, Skill, Skills, StatType, Status, StatusCooldownType, StatusType};
use crossterm::event::{read as read_event, Event, KeyCode};
use std::cell::RefCell;
use std::io::{stdout, Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Widget},
    Terminal,
};

type Term = Terminal<CrosstermBackend<Stdout>>;

pub enum MainMenuAction {
    Play,
    Edit,
    Quit,
}

pub enum GameAction {
    UseSkill,
    AddStatus,
    DrainStatusAttacking,
    DrainStatusAttacked,
    ManageMoney,
    ClearStatuses,
    ResetSkillsCD,
    MakeTurn,
    SkipTurn,
    NextPlayerPick,
    Quit,
}

pub enum CharacterMenuAction {
    Add,
    Edit(usize),
    Delete(usize),
    Quit,
}

#[derive(Copy, Clone)]
pub enum CharacterMenuMode {
    View(usize),
    Add,
    Edit(usize),
}

enum StatusBarType {
    Normal,
    Error,
}

#[derive(Copy, Clone)]
enum PlayerField {
    Name,
    Stat(StatType),
    SkillName(usize),
    SkillCD(usize),
}

impl PlayerField {
    fn next(&self) -> PlayerField {
        match self {
            PlayerField::Name => PlayerField::Stat(StatType::Strength),
            PlayerField::Stat(stat) => match stat {
                StatType::Strength => PlayerField::Stat(StatType::Dexterity),
                StatType::Dexterity => PlayerField::Stat(StatType::Poise),
                StatType::Poise => PlayerField::Stat(StatType::Wisdom),
                StatType::Wisdom => PlayerField::Stat(StatType::Intelligence),
                StatType::Intelligence => PlayerField::Stat(StatType::Charisma),
                StatType::Charisma => PlayerField::SkillName(0),
            },
            PlayerField::SkillName(i) => PlayerField::SkillCD(*i),
            PlayerField::SkillCD(i) => PlayerField::SkillName(*i + 1),
        }
    }

    fn prev(&self) -> PlayerField {
        match self {
            PlayerField::Name => PlayerField::Name,
            PlayerField::Stat(stat) => match stat {
                StatType::Strength => PlayerField::Name,
                StatType::Dexterity => PlayerField::Stat(StatType::Strength),
                StatType::Poise => PlayerField::Stat(StatType::Dexterity),
                StatType::Wisdom => PlayerField::Stat(StatType::Poise),
                StatType::Intelligence => PlayerField::Stat(StatType::Wisdom),
                StatType::Charisma => PlayerField::Stat(StatType::Intelligence),
            },
            PlayerField::SkillName(i) => {
                if *i == 0 {
                    PlayerField::Stat(StatType::Charisma)
                } else {
                    PlayerField::SkillCD(*i - 1)
                }
            }
            PlayerField::SkillCD(i) => PlayerField::SkillName(*i),
        }
    }
}

pub struct Tui {
    term: RefCell<Term>,
}

impl Tui {
    pub fn new() -> Tui {
        crossterm::terminal::enable_raw_mode().unwrap();
        Tui {
            term: RefCell::new(Terminal::new(CrosstermBackend::new(stdout())).unwrap()),
        }
    }

    fn get_window_size(&self, window: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(1)].as_ref())
            .split(window)
    }

    fn stylize_statusbar<'a, T: Into<Text<'a>>>(text: T, sbtype: StatusBarType) -> Paragraph<'a> {
        let style = match sbtype {
            StatusBarType::Normal => Style::default().bg(Color::Cyan).fg(Color::Black),
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

    pub fn messagebox_with_options(
        &self,
        desc: &str,
        options: &[&str],
        is_vertical: bool,
    ) -> usize {
        let width = {
            let desc_width = desc.len() as u16 + 4;
            let button_width = {
                if !is_vertical {
                    // add all button text together
                    let mut tmp = 0;
                    for option in options.iter() {
                        tmp += option.chars().count() as u16;
                    }

                    tmp += 4;
                    tmp
                } else {
                    // find the longest button text
                    options.iter().fold(0, |acc, item| {
                        let len = item.chars().count();
                        if len > acc {
                            len
                        } else {
                            acc
                        }
                    }) as u16
                        + 4
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

        let mut currently_selected: usize = 0;
        loop {
            self.term
                .borrow_mut()
                .draw(|frame| {
                    let block_rect = Tui::get_centered_box(frame.size(), width, height);
                    let (desc_rect, buttons_rect) =
                        Tui::get_messagebox_text_input_locations(block_rect);

                    let block = Block::default().borders(Borders::ALL);
                    let desc = Paragraph::new(desc).alignment(Alignment::Center);
                    frame.render_widget(block.clone(), block_rect);
                    frame.render_widget(desc, desc_rect);

                    if !is_vertical {
                        const OFFSET_BETWEEN_BUTTONS: u16 = 3;
                        let buttons_rect = {
                            let offset = {
                                let mut tmp = buttons_rect.width;
                                for option in options.iter() {
                                    tmp -= option.chars().count() as u16;
                                }
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
                            let button_style = if i == currently_selected {
                                Style::default().bg(Color::White).fg(Color::Black)
                            } else {
                                Style::default()
                            };

                            let button = Paragraph::new(*option).style(button_style);

                            let rect = {
                                let mut tmp = buttons_rect;
                                tmp.width = option.chars().count() as u16;
                                if i > 0 {
                                    tmp.x += options[i - 1].len() as u16;
                                    tmp.x += OFFSET_BETWEEN_BUTTONS;
                                }

                                tmp
                            };

                            frame.render_widget(button, rect);
                        }
                    } else {
                        for (i, option) in options.iter().enumerate() {
                            let option_len = option.chars().count() as u16;
                            let offset = (width - option_len) / 2;
                            let rect = {
                                let mut tmp = buttons_rect.clone();
                                tmp.y += i as u16;
                                tmp.width = option_len;
                                tmp
                            };

                            let button_style = if i == currently_selected {
                                Style::default().bg(Color::White).fg(Color::Black)
                            } else {
                                Style::default()
                            };

                            let button = Paragraph::new(*option).style(button_style);
                            frame.render_widget(button, rect);
                        }
                    }
                })
                .unwrap();

            match read_event().unwrap() {
                Event::Key(key) => match key.code {
                    KeyCode::Enter => {
                        return currently_selected;
                    }
                    KeyCode::Right => {
                        if !is_vertical {
                            if currently_selected >= options.len() - 1 {
                                currently_selected = 0;
                            } else {
                                currently_selected += 1;
                            }
                        }
                    }
                    KeyCode::Left => {
                        if !is_vertical {
                            if currently_selected == 0 {
                                currently_selected = options.len() - 1;
                            } else {
                                currently_selected -= 1;
                            }
                        }
                    }
                    KeyCode::Down => {
                        if is_vertical {
                            if currently_selected >= options.len() - 1 {
                                currently_selected = 0;
                            } else {
                                currently_selected += 1;
                            }
                        }
                    }
                    KeyCode::Up => {
                        if is_vertical {
                            if currently_selected == 0 {
                                currently_selected = options.len() - 1;
                            } else {
                                currently_selected -= 1;
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn messagebox_with_input_field(&self, desc: &str) -> String {
        let width = desc.len() as u16 + 4;
        let height = 7;
        let mut buffer = String::new();

        loop {
            self.term
                .borrow_mut()
                .draw(|frame| {
                    let block_rect = Tui::get_centered_box(frame.size(), width, height);
                    let (desc_rect, input_rect) =
                        Tui::get_messagebox_text_input_locations(block_rect);

                    let block = Block::default().borders(Borders::ALL);
                    let desc = Paragraph::new(desc).alignment(Alignment::Center);
                    let input = Paragraph::new(buffer.as_str());
                    frame.render_widget(block.clone(), block_rect);
                    frame.render_widget(desc, desc_rect);
                    frame.render_widget(input, input_rect);
                })
                .unwrap();

            match read_event().unwrap() {
                Event::Key(key) => match key.code {
                    KeyCode::Char(ch) => buffer.push(ch),
                    KeyCode::Backspace => {
                        buffer.pop();
                    }
                    KeyCode::Enter => {
                        return buffer;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn messagebox_yn(&self, desc: &str) -> bool {
        match self.messagebox_with_options(desc, &["Yes", "No"], false) {
            0 => true,
            _ => false,
        }
    }

    pub fn messagebox(&self, desc: &str) {
        self.messagebox_with_options(desc, &["OK"], false);
    }

    pub fn draw_main_menu(&mut self) -> MainMenuAction {
        self.term.borrow_mut().clear().unwrap();
        loop {
            self.term
                .borrow_mut()
                .draw(|frame| {
                    let layout = self.get_window_size(frame.size());

                    let items = [
                        ListItem::new("1. Start game"),
                        ListItem::new("2. Manage characters"),
                        ListItem::new("3. Save and quit"),
                    ];
                    let list = List::new(items);

                    frame.render_widget(list, layout[0]);
                    frame.render_widget(
                        Tui::stylize_statusbar(
                            format!("dnd-gm-helper v{}", env!("CARGO_PKG_VERSION")),
                            StatusBarType::Normal,
                        ),
                        layout[1],
                    );
                })
                .unwrap();

            match read_event().unwrap() {
                Event::Key(key) => match key.code {
                    KeyCode::Esc => {
                        if self.messagebox_yn("Are you sure you want to quit?") {
                            return MainMenuAction::Quit;
                        }
                    }
                    KeyCode::Char(ch) => match ch {
                        '1' => return MainMenuAction::Play,
                        '2' => return MainMenuAction::Edit,
                        '3' | 'q' => {
                            if self.messagebox_yn("Are you sure you want to quit?") {
                                return MainMenuAction::Quit;
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn draw_game(&mut self, player: &Player) -> GameAction {
        loop {
            self.term
                .borrow_mut()
                .draw(|frame| {
                    let player_stats = Tui::player_stats_table(player, None);
                    let delimiter = Span::raw(" | ");
                    let style_underlined = Style::default().add_modifier(Modifier::UNDERLINED);
                    let statusbar_text = Spans::from(vec![
                        "Use ".into(),
                        Span::styled("s", style_underlined),
                        "kill".into(),
                        delimiter.clone(),
                        Span::styled("A", style_underlined),
                        "dd status".into(),
                        delimiter.clone(),
                        "Drain status (a".into(),
                        Span::styled("f", style_underlined),
                        "ter attacking".into(),
                        ", ".into(),
                        "after ".into(),
                        Span::styled("g", style_underlined),
                        "etting attacked)".into(),
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
                        Span::styled("P", style_underlined),
                        "ick next pl.".into(),
                        delimiter.clone(),
                        Span::styled("Q", style_underlined),
                        "uit".into(),
                    ]);
                    let layout = self.get_window_size(frame.size());

                    frame.render_widget(player_stats, layout[0]);
                    frame.render_widget(
                        Tui::stylize_statusbar(statusbar_text, StatusBarType::Normal),
                        layout[1],
                    );
                })
                .unwrap();

            match read_event().unwrap() {
                Event::Key(key) => match key.code {
                    KeyCode::Char(ch) => match ch {
                        's' => return GameAction::UseSkill,
                        'a' => return GameAction::AddStatus,
                        'b' => return GameAction::DrainStatusAttacking,
                        'n' => return GameAction::DrainStatusAttacked,
                        'c' => return GameAction::ClearStatuses,
                        'v' => return GameAction::ResetSkillsCD,
                        'm' => return GameAction::ManageMoney,
                        ' ' => return GameAction::MakeTurn,
                        'p' => return GameAction::SkipTurn,
                        'o' => return GameAction::NextPlayerPick,
                        'q' => return GameAction::Quit,
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn player_stats_table(player: &Player, selected: Option<PlayerField>) -> impl Widget + '_ {
        let selected_style = Style::default().bg(Color::White).fg(Color::Black);
        let mut rows = vec![
            Row::new(["Name", player.name.as_str()]).style(
                if let Some(PlayerField::Name) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                },
            ),
            Row::new(["Stats"]),
            // TODO: mb use a slice instead
            Row::new::<Vec<Cell>>(vec![
                "Strength".into(),
                player.stats.strength.to_string().into(),
            ])
            .style(
                if let Some(PlayerField::Stat(StatType::Strength)) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                },
            ),
            Row::new::<Vec<Cell>>(vec![
                "Dexterity".into(),
                player.stats.dexterity.to_string().into(),
            ])
            .style(
                if let Some(PlayerField::Stat(StatType::Dexterity)) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                },
            ),
            Row::new::<Vec<Cell>>(vec!["Poise".into(), player.stats.poise.to_string().into()])
                .style(if let Some(PlayerField::Stat(StatType::Poise)) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                }),
            Row::new::<Vec<Cell>>(vec![
                "Wisdom".into(),
                player.stats.wisdom.to_string().into(),
            ])
            .style(
                if let Some(PlayerField::Stat(StatType::Wisdom)) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                },
            ),
            Row::new::<Vec<Cell>>(vec![
                "Intelligence".into(),
                player.stats.intelligence.to_string().into(),
            ])
            .style(
                if let Some(PlayerField::Stat(StatType::Intelligence)) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                },
            ),
            Row::new::<Vec<Cell>>(vec![
                "Charisma".into(),
                player.stats.charisma.to_string().into(),
            ])
            .style(
                if let Some(PlayerField::Stat(StatType::Charisma)) = selected {
                    selected_style.clone()
                } else {
                    Style::default()
                },
            ),
            Row::new(["Skills"]),
        ];

        for (i, skill) in player.skills.iter().enumerate() {
            rows.push(
                Row::new::<Vec<Cell>>(vec![
                    "Name".into(),
                    skill.name.as_str().into(),
                    "CD".into(),
                    skill.cooldown.to_string().into(),
                    "Available after".into(),
                    skill.available_after.to_string().into(),
                ])
                .style(
                    if let Some(PlayerField::SkillName(current_skill_num))
                    | Some(PlayerField::SkillCD(current_skill_num)) = selected
                    {
                        if current_skill_num == i {
                            selected_style.clone()
                        } else {
                            Style::default()
                        }
                    } else {
                        Style::default()
                    },
                ),
            );
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

        Table::new(rows).widths(
            [
                Constraint::Length(10),
                Constraint::Percentage(25),
                Constraint::Length(10),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
    }

    pub fn choose_skill(&self, skills: &Skills) -> Option<u32> {
        Some(
            self.messagebox_with_options(
                "Select skill",
                skills
                    .iter()
                    .map(|skill| skill.name.as_str())
                    .collect::<Vec<&str>>()
                    .as_slice(),
                true,
            ) as u32,
        )
    }

    pub fn choose_status(&self) -> Option<Status> {
        let status_list = [
            "#1 Discharge",
            "#2 Fire Attack",
            "#3 Fire Shield",
            "#4 Ice Shield",
            "#5 Blizzard",
            "#6 Fusion",
            "#7 Luck",
            "#8 Knockdown",
            "#9 Poison",
            "#0 Stun",
        ];

        let status_type = match self.messagebox_with_options("Choose a status", &status_list, true)
        {
            0 => StatusType::Discharge,
            1 => StatusType::FireAttack,
            2 => StatusType::FireShield,
            3 => StatusType::IceShield,
            4 => StatusType::Blizzard,
            5 => StatusType::Fusion,
            6 => StatusType::Luck,
            7 => StatusType::Knockdown,
            8 => StatusType::Poison,
            9 => StatusType::Stun,
            _ => unreachable!(),
        };

        let status_cooldown_type = match self.messagebox_with_options(
            "Status cooldown type",
            &["Normal", "On getting attacked", "On attacking"],
            true,
        ) {
            0 => StatusCooldownType::Normal,
            1 => StatusCooldownType::Attacked,
            2 => StatusCooldownType::Attacking,
            _ => unreachable!(),
        };

        let duration = loop {
            match self
                .messagebox_with_input_field("Status duration")
                .parse::<u32>()
            {
                Ok(num) => break num,
                Err(_) => self.messagebox("Not a valid number"),
            }
        };

        Some(Status {
            status_type,
            status_cooldown_type,
            duration,
        })
    }

    pub fn get_money_amount(&self) -> i64 {
        loop {
            let input =
                self.messagebox_with_input_field("Add or remove money");

            let input: i64 = match input.parse() {
                Ok(num) => num,
                Err(_) => {
                    self.messagebox(
                        format!("{} is not a valid input. Good examples: 500, -68", input).as_str(),
                    );
                    continue;
                }
            };

            return input;
        }
    }

    pub fn pick_player<'a>(&self, players: &'a Players) -> &'a Player {
        players.get(self.messagebox_with_options("Pick a player", players.iter().map(|x| x.name.as_str()).collect::<Vec<&str>>().as_slice(), true)).unwrap()
    }

    // TODO: separate most logic out of the UI and into the backend
    // don't mix frontend and backend stupid
    pub fn draw_character_menu(
        &mut self,
        mode: CharacterMenuMode,
        players: &mut Players,
    ) -> Option<CharacterMenuAction> {
        let mut add_mode_current_field: Option<PlayerField> = None;
        let mut add_mode_buffer: Option<String> = None;

        if let CharacterMenuMode::Add | CharacterMenuMode::Edit(_) = mode {
            // add an empty entry we are going to modify next
            if let CharacterMenuMode::Add = mode {
                players.push(Player::default());
            }

            add_mode_current_field = Some(PlayerField::Name);
        }

        let mut errors: Vec<String> = Vec::new();
        let mut player_list_state = ListState::default();
        loop {
            // stupid workaround to match both Add and Edit(i) at once
            if let (CharacterMenuMode::Add, i) | (CharacterMenuMode::Edit(i), _) = (mode, 0) {
                let current_player = if let CharacterMenuMode::Add = mode {
                    players.last_mut().unwrap()
                } else {
                    players.get_mut(i).unwrap()
                };

                match add_mode_current_field {
                    Some(field) => {
                        match field {
                            PlayerField::Name => {
                                // TODO: don't copy twice stupid
                                if let None = add_mode_buffer {
                                    add_mode_buffer = Some(current_player.name.clone());
                                }
                                current_player.name = add_mode_buffer.as_ref().unwrap().clone()
                            }
                            PlayerField::Stat(stat) => {
                                let current_stat = match stat {
                                    StatType::Strength => &mut current_player.stats.strength,
                                    StatType::Dexterity => &mut current_player.stats.dexterity,
                                    StatType::Poise => &mut current_player.stats.poise,
                                    StatType::Wisdom => &mut current_player.stats.wisdom,
                                    StatType::Intelligence => {
                                        &mut current_player.stats.intelligence
                                    }
                                    StatType::Charisma => &mut current_player.stats.charisma,
                                };
                                if let None = add_mode_buffer {
                                    add_mode_buffer = Some(current_stat.to_string());
                                }
                                match add_mode_buffer.as_ref().unwrap().parse::<i64>() {
                                    Ok(num) => *current_stat = num,
                                    Err(_) => errors.push(String::from(format!(
                                        "Not a valid number(s): {}",
                                        add_mode_buffer.as_ref().unwrap()
                                    ))),
                                }
                            }
                            // TODO: make that look nicer
                            // currently a new skill just appears out of nowhere when you start typing
                            PlayerField::SkillName(i) => {
                                if let None = current_player.skills.get(i) {
                                    current_player.skills.push(Skill::default());
                                }
                                let skill = &mut current_player.skills[i];

                                if let None = add_mode_buffer {
                                    add_mode_buffer = Some(skill.name.clone());
                                }

                                skill.name = add_mode_buffer.as_ref().unwrap().clone();
                            }
                            PlayerField::SkillCD(i) => {
                                if let None = current_player.skills.get(i) {
                                    current_player.skills.push(Skill::default());
                                }
                                let skill = &mut current_player.skills[i];

                                if let None = add_mode_buffer {
                                    add_mode_buffer = Some(skill.cooldown.to_string());
                                }
                                match add_mode_buffer.as_ref().unwrap().parse::<u32>() {
                                    Ok(num) => skill.cooldown = num,
                                    Err(_) => errors.push(String::from(format!(
                                        "Not a valid number(s): {}",
                                        add_mode_buffer.as_ref().unwrap()
                                    ))),
                                }
                            }
                        }
                    }
                    None => return None,
                }
            }

            let mut player_list_items = Vec::new();
            for player in players.iter() {
                player_list_items.push(ListItem::new(player.name.clone()));
            }

            // default currently selected entry for each mode
            player_list_state.select(match mode {
                CharacterMenuMode::View(i) => {
                    // select the first entry if the list isn't empty
                    if player_list_items.len() > 0 {
                        if let None = player_list_state.selected() {
                            Some(i)
                        } else {
                            player_list_state.selected()
                        }
                    } else {
                        player_list_state.selected()
                    }
                }
                CharacterMenuMode::Add => {
                    // always select the last/currenty in process of adding entry
                    Some(player_list_items.len() - 1)
                }
                CharacterMenuMode::Edit(i) => Some(i),
            });

            self.term
                .borrow_mut()
                .draw(|frame| {
                    let layout = self.get_window_size(frame.size());
                    let tables = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(layout[0]);

                    let player_list = List::new(player_list_items.clone())
                        .block(Block::default().title("Players").borders(Borders::ALL))
                        .highlight_symbol(">> ");

                    let style_underlined = Style::default().add_modifier(Modifier::UNDERLINED);
                    let delimiter = Span::raw(" | ");

                    if errors.is_empty() {
                        let statusbar_text = match mode {
                            CharacterMenuMode::View(_) => {
                                Spans::from(vec![
                                    " ".into(),
                                    Span::styled("A", style_underlined),
                                    "dd".into(),
                                    delimiter.clone(),
                                    Span::styled("E", style_underlined),
                                    "dit".into(),
                                    delimiter.clone(),
                                    Span::styled("D", style_underlined),
                                    "elete".into(),
                                    delimiter.clone(),
                                    Span::styled("Q", style_underlined),
                                    "uit".into(),
                                ])
                            }
                            CharacterMenuMode::Add => {
                                Spans::from("Add mode. Press Enter, Up, or down arrows to navigate | ESC to quit")
                            }
                            CharacterMenuMode::Edit(_) => {
                                Spans::from("Edit mode. Press Up or down arrows to navigate | Enter or ESC to quit")
                            }
                        };
                        frame.render_widget(
                            Tui::stylize_statusbar(statusbar_text, StatusBarType::Normal),
                            layout[1],
                        );
                    } else {
                        frame.render_widget(
                            Tui::stylize_statusbar(errors.pop().unwrap(), StatusBarType::Error),
                            layout[1],
                        );
                    }

                    frame.render_stateful_widget(player_list, tables[0], &mut player_list_state);

                    if let Some(num) = player_list_state.selected() {
                        /*
                        let paragraph = Paragraph::new(Tui::print_player_stats(&players[num]))
                            .block(Block::default().title("Player stats").borders(Borders::ALL));

                        frame.render_widget(paragraph, tables[1]);
                        */
                        frame.render_widget(Tui::player_stats_table(&players[num], add_mode_current_field), tables[1]);
                    }
                })
                .unwrap();

            if let Event::Key(key) = read_event().unwrap() {
                match mode {
                    CharacterMenuMode::View(_) => match key.code {
                        KeyCode::Char(ch) => match ch {
                            'a' => return Some(CharacterMenuAction::Add),
                            'e' => {
                                if let Some(i) = player_list_state.selected() {
                                    return Some(CharacterMenuAction::Edit(i));
                                }
                            }
                            'd' => {
                                if let Some(i) = player_list_state.selected() {
                                    return Some(CharacterMenuAction::Delete(i));
                                }
                            }
                            'q' => return Some(CharacterMenuAction::Quit),
                            _ => (),
                        },
                        KeyCode::Down => {
                            let i = match player_list_state.selected() {
                                Some(i) => {
                                    if i >= player_list_items.len() - 1 {
                                        Some(0)
                                    } else {
                                        Some(i + 1)
                                    }
                                }
                                None => {
                                    if player_list_items.len() > 1 {
                                        Some(0)
                                    } else {
                                        None
                                    }
                                }
                            };
                            player_list_state.select(i);
                        }
                        KeyCode::Up => {
                            let i = match player_list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        Some(player_list_items.len() - 1)
                                    } else {
                                        Some(i - 1)
                                    }
                                }
                                None => {
                                    if player_list_items.len() > 1 {
                                        Some(0)
                                    } else {
                                        None
                                    }
                                }
                            };
                            player_list_state.select(i);
                        }
                        _ => (),
                    },
                    CharacterMenuMode::Add | CharacterMenuMode::Edit(_) => match key.code {
                        KeyCode::Char(ch) => {
                            if let None = add_mode_buffer {
                                add_mode_buffer = Some(String::new());
                            }
                            add_mode_buffer.as_mut().unwrap().push(ch);
                        }
                        KeyCode::Up => {
                            *add_mode_current_field.as_mut().unwrap() =
                                add_mode_current_field.as_ref().unwrap().prev();
                            add_mode_buffer = None;
                        }
                        KeyCode::Down => {
                            *add_mode_current_field.as_mut().unwrap() =
                                add_mode_current_field.as_ref().unwrap().next();
                            add_mode_buffer = None;
                        }
                        KeyCode::Backspace => {
                            add_mode_buffer.as_mut().unwrap().pop();
                        }
                        KeyCode::Enter => {
                            if let CharacterMenuMode::Add = mode {
                                let mut next =
                                    Some(add_mode_current_field.as_ref().unwrap().next());

                                // if pressed Enter with an empty buffer when adding skills - the last item -> done
                                if let Some(PlayerField::SkillName(current_skill_num)) =
                                    add_mode_current_field
                                {
                                    if add_mode_buffer.as_ref().unwrap().is_empty() {
                                        // TODO: somehow avoid nest matching mode twice
                                        let current_player_num = match mode {
                                            CharacterMenuMode::Add => players.len() - 1,
                                            CharacterMenuMode::Edit(num) => num,
                                            _ => unreachable!(),
                                        };
                                        players[current_player_num]
                                            .skills
                                            .remove(current_skill_num);
                                        next = None;
                                    }
                                // don't assume a default skill cd, just don't do anything
                                } else if let Some(PlayerField::SkillCD(_)) = add_mode_current_field
                                {
                                    if add_mode_buffer.as_ref().unwrap().is_empty() {
                                        continue;
                                    }
                                }

                                add_mode_current_field = next;
                                add_mode_buffer = None;
                            } else if let CharacterMenuMode::Edit(_) = mode {
                                return None;
                            }
                        }
                        KeyCode::Esc => {
                            if let CharacterMenuMode::Add = mode {
                                players.pop();
                            }
                            return None;
                        }
                        _ => (),
                    },
                }
            }
        }
    }
}
