// TMP
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::{Player, Players, Skill, Skills, Status, StatusCooldownType, StatusType};
use crossterm::event::{read as read_event, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::cell::RefCell;
use std::io::{stdout, Stdout, Write};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Widget},
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

    fn stylize_statusbar<'a, T: Into<Text<'a>>>(text: T) -> Paragraph<'a> {
        Paragraph::new(text.into()).style(Style::default().bg(Color::Cyan).fg(Color::Black))
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

    pub fn draw_main_menu(&mut self) -> MainMenuAction {
        let items = [
            ListItem::new("1. Start game"),
            ListItem::new("2. Manage characters"),
            ListItem::new("3. Exit"),
        ];
        let list = List::new(items);
        /*
        self.draw(
            vec![list],
        );
        */

        self.term.borrow_mut().clear().unwrap();
        self.term
            .borrow_mut()
            .draw(|frame| {
                let layout = self.get_window_size(frame.size());

                frame.render_widget(list, layout[0]);
                frame.render_widget(
                    Tui::stylize_statusbar(format!("dnd-gm-helper v{}", env!("CARGO_PKG_VERSION"))),
                    layout[1],
                );
            })
            .unwrap();

        loop {
            match Tui::get_input_char() {
                '1' => return MainMenuAction::Play,
                '2' => return MainMenuAction::Edit,
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
                frame.render_widget(Tui::stylize_statusbar(statusbar_text), layout[1]);
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

    pub fn draw_character_menu(&mut self, players: &Players) -> CharacterMenuAction {
        let mut player_list_items = Vec::new();
        let mut player_list_state = ListState::default();
        player_list_state.select(Some(0));

        for player in players {
            player_list_items.push(ListItem::new(player.name.clone()));
        }

        loop {
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

                    let player_stats = Paragraph::new(Tui::print_player_stats(
                        &players[player_list_state.selected().unwrap()],
                    ))
                    .block(Block::default().title("Player stats").borders(Borders::ALL));
                    let statusbar_text = "Add: a, Edit: e, Delete: d, Quit: q";

                    frame.render_stateful_widget(player_list, tables[0], &mut player_list_state);
                    frame.render_widget(player_stats, tables[1]);
                    frame.render_widget(Tui::stylize_statusbar(statusbar_text), layout[1]);
                })
                .unwrap();

            loop {
                if let Event::Key(key) = read_event().unwrap() {
                    match key.code {
                        KeyCode::Char(ch) => match ch {
                            'a' => return CharacterMenuAction::Add,
                            'e' => {
                                let input: usize = loop {
                                    match Tui::get_input_string().parse() {
                                        Ok(num) => break num,
                                        Err(_) => Tui::err("Not a valid number"),
                                    };
                                };
                                return CharacterMenuAction::Edit(input);
                            }
                            'd' => {
                                let input: usize = loop {
                                    match Tui::get_input_string().parse() {
                                        Ok(num) => break num,
                                        Err(_) => Tui::err("Not a valid number"),
                                    };
                                };
                                return CharacterMenuAction::Delete(input);
                            }
                            'q' => return CharacterMenuAction::Quit,
                            _ => (),
                        },
                        KeyCode::Down => {
                            let i = match player_list_state.selected() {
                                Some(i) => {
                                    if i >= player_list_items.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            player_list_state.select(Some(i));
                            break;
                        }
                        KeyCode::Up => {
                            let i = match player_list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        player_list_items.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            player_list_state.select(Some(i));
                            break;
                        }
                        _ => (),
                    }
                }
            }
        }
    }

    pub fn edit_player(&mut self, player: Option<Player>) -> Player {
        unimplemented!();
        /*
        disable_raw_mode().unwrap();
        fn get_text(_term: &mut Term, old_value: String, stat_name: &str) -> String {
            if !old_value.is_empty() {
                println!("Old {}: {}. Press enter to skip", stat_name, old_value);
            }
            print!("{}: ", stat_name);
            stdout().flush().unwrap();
            let input = Tui::get_input_string().trim().to_string();
            if !old_value.is_empty() && input.is_empty() {
                return old_value;
            }
            input
        }

        fn get_stat_num(term: &mut Term, old_value: i64, stat_name: &str) -> i64 {
            loop {
                if old_value != 0 {
                    println!("Old {}: {}. Press enter to skip", stat_name, old_value);
                }
                print!("{}: ", stat_name);
                stdout().flush().unwrap();
                let input = Tui::get_input_string().trim().to_string();
                if old_value != 0 && input.is_empty() {
                    return old_value;
                }
                match input.parse() {
                    Ok(num) => return num,
                    Err(_) => err(term, "Not a valid number"),
                }
            }
        }

        fn err(_term: &mut Term, text: &str) {
            print!("{}", text);
            stdout().flush().unwrap();
            //stdout().flush();
            read_event().unwrap();
        }

        //let mut player: Player = Default::default();
        let mut player: Player = player.unwrap_or_default();

        player.name = get_text(&mut self.term, player.name, "Name");

        println!("Stats:");
        player.stats.strength = get_stat_num(&mut self.term, player.stats.strength, "Strength");
        player.stats.dexterity = get_stat_num(&mut self.term, player.stats.dexterity, "Dexterity");
        player.stats.poise = get_stat_num(&mut self.term, player.stats.poise, "Poise");
        player.stats.wisdom = get_stat_num(&mut self.term, player.stats.wisdom, "Wisdom");
        player.stats.intelligence =
            get_stat_num(&mut self.term, player.stats.intelligence, "Intelligence");
        player.stats.charisma = get_stat_num(&mut self.term, player.stats.charisma, "Charisma");

        // edit the existing skills first
        if !player.skills.is_empty() {
            for (i, skill) in player.skills.iter_mut().enumerate() {
                skill.name = get_text(&mut self.term, skill.name.clone(), &format!("Skill #{}", i));
                // TODO: parse i64 to u32 correctly
                skill.cooldown =
                    get_stat_num(&mut self.term, skill.cooldown as i64, "Cooldown") as u32;
                print!("Reset existing cooldown to 0? ");
                stdout().flush().unwrap();
                match Tui::get_input_char() {
                    'y' => skill.available_after = 0,
                    _ => (),
                }
            }
        }

        println!("Add new skills?");
        match Tui::get_input_char() {
            'y' => loop {
                print!("Skill name (\"q\" to quit): ");
                stdout().flush().unwrap();
                let name = Tui::get_input_string().trim().to_string();
                if name == "q" {
                    break;
                }
                print!("Skill cooldown: ");
                stdout().flush().unwrap();
                let cd = loop {
                    match Tui::get_input_string().trim().parse::<u32>() {
                        Ok(num) => break num,
                        Err(_) => err(&mut self.term, "Not a valid number"),
                    };
                };
                player.skills.push(Skill::new(name, cd));
            },
            _ => (),
        }

        player.money = get_stat_num(&mut self.term, player.money, "Money");
        enable_raw_mode().unwrap();
        player
    */
    }
}
