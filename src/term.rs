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
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
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

    fn draw<T: Widget>(&self, widget: T, statusbar: Spans) {
        self.term.borrow_mut().clear().unwrap();
        self.term
            .borrow_mut()
            .draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10), Constraint::Length(1)].as_ref())
                    .split(frame.size());

                frame.render_widget(widget, chunks[0]);

                let paragraph = Paragraph::new(statusbar)
                    .style(Style::default().bg(Color::Cyan).fg(Color::Black));
                frame.render_widget(paragraph, chunks[1]);
            })
            .unwrap();
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
        self.draw(
            list,
            Span::raw(format!("dnd-gm-helper v{}", env!("CARGO_PKG_VERSION"))).into(),
        );

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
        unimplemented!();
        /*
        self.term.clear().unwrap();
        self.term.set_cursor(0, 0).unwrap();
        self.draw_player_stats(player);
        disable_raw_mode().unwrap();
        println!("Use skill: s|Add status: a|Drain status after attacking: b, after getting attacked: n|Reset statuses: c, skill CD: v|Manage money: m|Next turn: \" \"|Skip turn: p|Pick next player: o|Quit game: q");
        enable_raw_mode().unwrap();

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
        */
    }

    pub fn draw_player_stats(&mut self, player: &Player) {
        disable_raw_mode().unwrap();
        println!("Name: {}", player.name);
        println!("Stats:");
        println!("....Strength: {}", player.stats.strength);
        println!("....Dexterity: {}", player.stats.dexterity);
        println!("....Poise: {}", player.stats.poise);
        println!("....Wisdom: {}", player.stats.wisdom);
        println!("....Intelligence: {}", player.stats.intelligence);
        println!("....Charisma: {}", player.stats.charisma);

        if !player.skills.is_empty() {
            println!("Skills:");
        }
        for skill in &player.skills {
            println!(
                "....{}. CD: {}. Available after {} moves",
                skill.name, skill.cooldown, skill.available_after
            );
        }

        if !player.statuses.is_empty() {
            println!("Statuses:");
        }
        for status in &player.statuses {
            println!(
                "....{:?}, Still active for {} moves",
                status.status_type, status.duration
            );
        }

        println!("Money: {}", player.money);
        enable_raw_mode().unwrap();
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
        unimplemented!();
        /*
        self.term.clear().unwrap();
        disable_raw_mode().unwrap();
        for (i, player) in players.iter().enumerate() {
            println!("#{}", i + 1);
            // TODO: replace with a table
            self.draw_player_stats(player);
        }
        println!("Add: a, Edit: e, Delete: d, Quit: q");
        enable_raw_mode().unwrap();

        loop {
            if let Event::Key(key) = read_event().unwrap() {
                if let KeyCode::Char(ch) = key.code {
                    match ch {
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
                    }
                }
            }
        }
        */
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
