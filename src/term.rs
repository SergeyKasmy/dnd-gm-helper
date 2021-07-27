// TMP
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::{Player, Players, Skill, Skills, StatType, Status, StatusCooldownType, StatusType};
use crossterm::event::{read as read_event, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::cell::RefCell;
use std::io::{stdout, Stdout, Write};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget},
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

pub struct Tui {
    term: RefCell<Term>,
}

impl Tui {
    pub fn new() -> Tui {
        enable_raw_mode().unwrap();

        let term = RefCell::new(Terminal::new(CrosstermBackend::new(stdout())).unwrap());

        Tui { term }
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

    fn get_input_char() -> char {
        enable_raw_mode().unwrap();
        loop {
            if let Event::Key(key) = read_event().unwrap() {
                if let KeyCode::Char(ch) = key.code {
                    return ch;
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn get_input_string() -> String {
        disable_raw_mode().unwrap();
        let mut input = String::new();

        loop {
            if let Event::Key(key) = read_event().unwrap() {
                match key.code {
                    KeyCode::Char(ch) => input.push(ch),
                    KeyCode::Enter => break,
                    _ => (),
                }
            }
        }

        enable_raw_mode().unwrap();
        input
    }

    // TODO: do something about this shit
    #[cfg(target_os = "windows")]
    fn get_input_string() -> String {
        enable_raw_mode().unwrap();
        let mut input = String::new();

        loop {
            if let Event::Key(key) = read_event().unwrap() {
                match key.code {
                    KeyCode::Char(ch) => {
                        input.push(ch);
                        disable_raw_mode().unwrap();
                        print!("{}", ch);
                        stdout().flush().unwrap();
                        enable_raw_mode().unwrap();
                    }
                    KeyCode::Enter => {
                        disable_raw_mode().unwrap();
                        print!("\n\r");
                        stdout().flush().unwrap();
                        enable_raw_mode().unwrap();
                        break;
                    }
                    _ => (),
                }
            }
        }

        input
    }

    pub fn err(text: &str) {
        disable_raw_mode().unwrap();
        eprintln!("{}", text);
        enable_raw_mode().unwrap();
    }

    // TODO: doesn't yet work, do something about it
    fn popup_with_options(&self, desc: &str, options: Vec<&str>) -> i32 {
        self.term
            .borrow_mut()
            .draw(|frame| {
                let layout_x = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Min(1),
                            Constraint::Length(7),
                            Constraint::Min(1),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());

                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Min(1),
                            Constraint::Length((desc.len() + 4) as u16),
                            Constraint::Min(1),
                        ]
                        .as_ref(),
                    )
                    .split(layout_x[1])[1];

                eprintln!("{:#?}\n{:#?}", layout_x, layout);

                let block = Block::default().borders(Borders::ALL);

                frame.render_widget(block, layout);
            })
            .unwrap();

        read_event().unwrap();
        0
    }

    pub fn draw_main_menu(&mut self) -> MainMenuAction {
        let items = [
            ListItem::new("1. Start game"),
            ListItem::new("2. Manage characters"),
            ListItem::new("3. Save and quit"),
        ];
        let list = List::new(items);

        self.term.borrow_mut().clear().unwrap();
        self.term
            .borrow_mut()
            .draw(|frame| {
                let layout = self.get_window_size(frame.size());

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

        loop {
            match Tui::get_input_char() {
                '1' => return MainMenuAction::Play,
                '2' => return MainMenuAction::Edit,
                // TODO: handle ESC
                '3' | 'q' => return MainMenuAction::Quit,
                _ => (),
            }
        }
    }

    pub fn draw_game(&mut self, player: &Player) -> GameAction {
        self.term
            .borrow_mut()
            .draw(|frame| {
                let player_stats = Tui::print_player_stats(player);
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

                frame.render_widget(Paragraph::new(player_stats), layout[0]);
                frame.render_widget(
                    Tui::stylize_statusbar(statusbar_text, StatusBarType::Normal),
                    layout[1],
                );
            })
            .unwrap();

        return loop {
            match Tui::get_input_char() {
                's' => break GameAction::UseSkill,
                'a' => break GameAction::AddStatus,
                'b' => break GameAction::DrainStatusAttacking,
                'n' => break GameAction::DrainStatusAttacked,
                'c' => break GameAction::ClearStatuses,
                'v' => break GameAction::ResetSkillsCD,
                'm' => break GameAction::ManageMoney,
                ' ' => break GameAction::MakeTurn,
                'p' => break GameAction::SkipTurn,
                'o' => break GameAction::NextPlayerPick,
                'q' => break GameAction::Quit,
                _ => (),
            }
        };
    }

    pub fn print_player_stats(player: &Player) -> Vec<Spans> {
        let mut out: Vec<Spans> = Vec::with_capacity(10);
        out.push(format!("Name: {}", player.name).into());
        out.push("Stats:".into());
        out.push(format!("....Strength: {}", player.stats.strength).into());
        out.push(format!("....Dexterity: {}", player.stats.dexterity).into());
        out.push(format!("....Poise: {}", player.stats.poise).into());
        out.push(format!("....Wisdom: {}", player.stats.wisdom).into());
        out.push(format!("....Intelligence: {}", player.stats.intelligence).into());
        out.push(format!("....Charisma: {}", player.stats.charisma).into());

        if !player.skills.is_empty() {
            out.push("Skills:".into());
        }
        for skill in &player.skills {
            out.push(
                format!(
                    "....{}. CD: {}. Available after {} moves",
                    skill.name, skill.cooldown, skill.available_after
                )
                .into(),
            );
        }

        if !player.statuses.is_empty() {
            out.push("Statuses:".into());
        }
        for status in &player.statuses {
            out.push(
                format!(
                    "....{:?}, Still active for {} moves",
                    status.status_type, status.duration
                )
                .into(),
            );
        }

        out.push(format!("Money: {}", player.money).into());

        out
    }

    pub fn choose_skill(skills: &Skills) -> Option<u32> {
        disable_raw_mode().unwrap();
        for (i, skill) in skills.iter().enumerate() {
            println!("#{}: {}", i + 1, skill.name);
        }
        enable_raw_mode().unwrap();

        loop {
            let input = Tui::get_input_char();
            match input {
                'q' => return None,
                _ => match input.to_digit(10) {
                    Some(num) => return Some(num),
                    None => {
                        Tui::err("Not a valid number");
                        return None;
                    }
                },
            }
        }
    }

    pub fn choose_status() -> Option<Status> {
        disable_raw_mode().unwrap();
        println!("Choose a status:");
        println!("Buffs:");
        println!("#1 Discharge");
        println!("#2 Fire Attack");
        println!("#3 Fire Shield");
        println!("#4 Ice Shield");
        println!("#5 Blizzard");
        println!("#6 Fusion");
        println!("#7 Luck");
        println!("Debuffs:");
        println!("#8 Knockdown");
        println!("#9 Poison");
        println!("#0 Stun");
        enable_raw_mode().unwrap();

        let status_type = loop {
            match Tui::get_input_char() {
                '1' => break StatusType::Discharge,
                '2' => break StatusType::FireAttack,
                '3' => break StatusType::FireShield,
                '4' => break StatusType::IceShield,
                '5' => break StatusType::Blizzard,
                '6' => break StatusType::Fusion,
                '7' => break StatusType::Luck,
                '8' => break StatusType::Knockdown,
                '9' => break StatusType::Poison,
                '0' => break StatusType::Stun,
                'q' => return None,
                _ => continue,
            };
        };

        disable_raw_mode().unwrap();
        println!("Status cooldown type (1 for normal, 2 for on getting attacked, 3 for attacking)");
        enable_raw_mode().unwrap();
        let status_cooldown_type = loop {
            match Tui::get_input_char().to_digit(10) {
                Some(num) => break num,
                None => Tui::err("Not a valid number"),
            }
        };

        let status_cooldown_type = match status_cooldown_type {
            1 => StatusCooldownType::Normal,
            2 => StatusCooldownType::Attacked,
            3 => StatusCooldownType::Attacking,
            _ => {
                Tui::err("Not a valid cooldown type");
                return None;
            }
        };

        disable_raw_mode().unwrap();
        print!("Enter status duration: ");
        stdout().flush().unwrap();
        enable_raw_mode().unwrap();
        let duration = loop {
            match Tui::get_input_string().trim().parse::<u32>() {
                Ok(num) => break num,
                Err(_) => Tui::err("Number out of bounds"),
            }
        };

        Some(Status {
            status_type,
            status_cooldown_type,
            duration,
        })
    }

    pub fn get_money_amount() -> i64 {
        print!("Add or remove money (use + or - before the amount): ");
        stdout().flush().unwrap();
        let input = Tui::get_input_string().trim().to_string();
        if input == "q" {
            return 0;
        }

        if input.len() < 2 {
            Tui::err(&format!(
                "{} is not a valid input. Good examples: +500, -69",
                input
            ));
            return 0;
        }

        let mut op = '.';
        let mut amount = String::new();

        for (i, ch) in input.chars().enumerate() {
            if i == 0 {
                op = ch;
            } else {
                amount.push(ch);
            }
        }

        let amount: i64 = match amount.parse() {
            Ok(num) => num,
            Err(_) => {
                Tui::err("Not a valid number");
                return 0;
            }
        };

        return match op {
            '-' => -amount,
            '+' | _ => amount,
        };
    }

    pub fn pick_player(players: &Players) -> &Player {
        disable_raw_mode().unwrap();
        for (i, player) in players.iter().enumerate() {
            println!("#{}. {}", i + 1, player.name);
        }
        enable_raw_mode().unwrap();

        loop {
            let num = loop {
                match Tui::get_input_char().to_digit(10) {
                    Some(num) => break (num - 1) as usize,
                    None => (),
                }
            };

            match players.get(num) {
                Some(player) => return player,
                None => (),
            }
        }
    }

    pub fn draw_character_menu(
        &mut self,
        mode: CharacterMenuMode,
        players: &mut Players,
    ) -> Option<CharacterMenuAction> {
        #[derive(Copy, Clone)]
        enum AddModeCurrentField {
            Name,
            Stat(StatType),
            SkillName(usize),
            SkillCD(usize),
            Done,
        }

        impl AddModeCurrentField {
            fn next(&self) -> AddModeCurrentField {
                match self {
                    AddModeCurrentField::Name => AddModeCurrentField::Stat(StatType::Strength),
                    AddModeCurrentField::Stat(stat) => match stat {
                        StatType::Strength => AddModeCurrentField::Stat(StatType::Dexterity),
                        StatType::Dexterity => AddModeCurrentField::Stat(StatType::Poise),
                        StatType::Poise => AddModeCurrentField::Stat(StatType::Wisdom),
                        StatType::Wisdom => AddModeCurrentField::Stat(StatType::Intelligence),
                        StatType::Intelligence => AddModeCurrentField::Stat(StatType::Charisma),
                        StatType::Charisma => AddModeCurrentField::SkillName(0),
                    },
                    AddModeCurrentField::SkillName(i) => AddModeCurrentField::SkillCD(*i),
                    AddModeCurrentField::SkillCD(i) => AddModeCurrentField::SkillName(*i + 1),
                    AddModeCurrentField::Done => AddModeCurrentField::Done,
                }
            }

            fn prev(&self) -> AddModeCurrentField {
                match self {
                    AddModeCurrentField::Name => AddModeCurrentField::Name,
                    AddModeCurrentField::Stat(stat) => match stat {
                        StatType::Strength => AddModeCurrentField::Name,
                        StatType::Dexterity => AddModeCurrentField::Stat(StatType::Strength),
                        StatType::Poise => AddModeCurrentField::Stat(StatType::Dexterity),
                        StatType::Wisdom => AddModeCurrentField::Stat(StatType::Poise),
                        StatType::Intelligence => AddModeCurrentField::Stat(StatType::Wisdom),
                        StatType::Charisma => AddModeCurrentField::Stat(StatType::Intelligence),
                    },
                    AddModeCurrentField::SkillName(i) => {
                        if *i == 0 {
                            AddModeCurrentField::Stat(StatType::Charisma)
                        } else {
                            AddModeCurrentField::SkillCD(*i - 1)
                        }
                    }
                    AddModeCurrentField::SkillCD(i) => AddModeCurrentField::SkillName(*i),
                    AddModeCurrentField::Done => AddModeCurrentField::Done,
                }
            }
        }

        let mut add_mode_current_field: Option<AddModeCurrentField> = None;
        let mut add_mode_buffer: Option<String> = None;

        if let CharacterMenuMode::Add | CharacterMenuMode::Edit(_) = mode {
            // add an empty entry we are going to modify next
            if let CharacterMenuMode::Add = mode {
                players.push(Player::default());
            }

            add_mode_current_field = Some(AddModeCurrentField::Name);
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

                match add_mode_current_field.as_ref().unwrap() {
                    AddModeCurrentField::Name => {
                        // TODO: don't copy twice stupid
                        if let None = add_mode_buffer {
                            add_mode_buffer = Some(current_player.name.clone());
                        }
                        current_player.name = add_mode_buffer.as_ref().unwrap().clone()
                    }
                    AddModeCurrentField::Stat(stat) => {
                        let current_stat = match stat {
                            StatType::Strength => &mut current_player.stats.strength,
                            StatType::Dexterity => &mut current_player.stats.dexterity,
                            StatType::Poise => &mut current_player.stats.poise,
                            StatType::Wisdom => &mut current_player.stats.wisdom,
                            StatType::Intelligence => &mut current_player.stats.intelligence,
                            StatType::Charisma => &mut current_player.stats.charisma,
                        };
                        if let None = add_mode_buffer {
                            add_mode_buffer = Some(current_stat.to_string());
                        }
                        match add_mode_buffer.as_ref().unwrap().parse::<i64>() {
                            Ok(num) => *current_stat = num,
                            Err(_) => errors
                                .push(String::from(format!("Not a valid number(s): {}", add_mode_buffer.as_ref().unwrap()))),
                        }
                    }
                    // TODO: make that look nicer
                    // currently a new skill just appears out of nowhere when you start typing
                    // TODO: pop a skill when pressed Enter with an empty name
                    AddModeCurrentField::SkillName(i) => {
                        if let None = current_player.skills.get(*i) {
                            current_player.skills.push(Skill::default());
                        }
                        let skill = &mut current_player.skills[*i];

                        if let None = add_mode_buffer {
                            add_mode_buffer = Some(skill.name.clone());
                        }

                        skill.name = add_mode_buffer.as_ref().unwrap().clone();
                    }
                    AddModeCurrentField::SkillCD(i) => {
                        if let None = current_player.skills.get(*i) {
                            current_player.skills.push(Skill::default());
                        }
                        let skill = &mut current_player.skills[*i];

                        if let None = add_mode_buffer {
                            *add_mode_buffer.as_mut().unwrap() = skill.cooldown.to_string();
                            add_mode_buffer = Some(skill.cooldown.to_string());
                        }
                        match add_mode_buffer.as_ref().unwrap().parse::<u32>() {
                            Ok(num) => skill.cooldown = num,
                            Err(_) => errors
                                .push(String::from(format!("Not a valid number(s): {}", add_mode_buffer.as_ref().unwrap()))),
                        }
                    }
                    AddModeCurrentField::Done => return None,
                }
            }

            let mut player_list_items = Vec::new();
            for player in players.iter() {
                player_list_items.push(ListItem::new(player.name.clone()));
            }

            // default currently selected entry for each mode
            match mode {
                CharacterMenuMode::View(i) => {
                    // select the first entry if the list isn't empty
                    if player_list_items.len() > 0 {
                        if let None = player_list_state.selected() {
                            player_list_state.select(Some(i));
                        }
                    }
                }
                CharacterMenuMode::Add => {
                    // always select the last/currenty in process of adding entry
                    player_list_state.select(Some(player_list_items.len() - 1));
                }
                CharacterMenuMode::Edit(i) => {
                    player_list_state.select(Some(i));
                }
            }

            self.term
                .borrow_mut()
                .draw(|frame| {
                    let layout = self.get_window_size(frame.size());
                    let tables = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(25), Constraint::Percentage(75)].as_ref(),
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
                                Spans::from("Add mode. Press ESC to quit")
                            }
                            CharacterMenuMode::Edit(_) => {
                                Spans::from("Edit mode. Press ESC to quit")
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
                        let paragraph = Paragraph::new(Tui::print_player_stats(&players[num]))
                            .block(Block::default().title("Player stats").borders(Borders::ALL));

                        frame.render_widget(paragraph, tables[1]);
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
                                let mut next = add_mode_current_field.as_ref().unwrap().next();
                                let current = add_mode_current_field.as_mut().unwrap();

                                // if pressed Enter with an empty buffer when adding skills - the last item -> done
                                if let AddModeCurrentField::SkillName(_) = current {
                                    if add_mode_buffer.as_ref().unwrap().is_empty() {
                                        next = AddModeCurrentField::Done;
                                    }
                                // don't assume a default skill cd, just don't do anything
                                } else if let AddModeCurrentField::SkillCD(_) = current {
                                    if add_mode_buffer.as_ref().unwrap().is_empty() {
                                        continue;
                                    }
                                }

                                *current = next;
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
