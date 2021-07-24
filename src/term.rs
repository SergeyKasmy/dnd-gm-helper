use crate::{Player, Players, Skill};
use crossterm::event::{read as read_event, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::{Stdout, Write};
use tui::{
    backend::CrosstermBackend,
    text::{Span, Spans},
    widgets::{List, ListItem, Paragraph},
    Terminal,
};

type Term = Terminal<CrosstermBackend<Stdout>>;

pub struct Tui {
    term: Term,
}

#[derive(Debug)]
pub enum MainMenuAction {
    Play,
    Edit,
    Quit,
}

pub enum CharacterMenuAction {
    Add,
    Edit(i32),
    Delete(i32),
    Quit,
}

impl Tui {
    pub fn new() -> Tui {
        crossterm::terminal::enable_raw_mode().unwrap();

        Tui {
            term: Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap(),
        }
    }

    fn get_input() -> String {
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

        input
    }

    pub fn err(text: &str) {
        disable_raw_mode().unwrap();
        eprintln!("{}", text);
        enable_raw_mode().unwrap();
    }

    pub fn draw_main_menu(&mut self) -> MainMenuAction {
        self.term.clear().unwrap();
        self.term
            .draw(|frame| {
                let items = [
                    ListItem::new(format!("{:?}", MainMenuAction::Play)),
                    ListItem::new(format!("{:?}", MainMenuAction::Edit)),
                    ListItem::new(format!("{:?}", MainMenuAction::Quit)),
                ];
                let list = List::new(items);
                frame.render_widget(list, frame.size());
            })
            .unwrap();

        loop {
            if let Event::Key(key) = read_event().unwrap() {
                if let KeyCode::Char(ch) = key.code {
                    match ch {
                        '1' => return MainMenuAction::Play,
                        '2' => return MainMenuAction::Edit,
                        '3' | 'q' => return MainMenuAction::Quit,
                        _ => (),
                    }
                }
            }
        }
    }

    pub fn draw_character_menu(&mut self, players: &Players) -> CharacterMenuAction {
        self.term.clear().unwrap();
        disable_raw_mode().unwrap();
        for (i, player) in players.iter().enumerate() {
            println!("#{}", i + 1);
            // TODO: replace with a table
            println!("Name: {}", player.name);
            println!("Class: {}", player.class);
            println!("Stats:");
            println!("....Strength: {}", player.stats.strength);
            println!("....Dexterity: {}", player.stats.dexterity);
            println!("....Poise: {}", player.stats.poise);
            println!("....Wisdom: {}", player.stats.wisdom);
            println!("....Intelligence: {}", player.stats.intelligence);
            println!("....Charisma: {}", player.stats.charisma);

            println!("Skills:");
            for skill in &player.skills {
                println!(
                    "....{}. CD: {}. Available after {} moves",
                    skill.name, skill.cooldown, skill.available_after
                );
            }

            println!("Statuses:");
            for status in &player.statuses {
                println!(
                    "....{:?}, Still active for {} moves",
                    status.status_type, status.duration
                );
            }

            println!("Money: {}", player.money);
        }
        println!("Add: a, Edit: e, Delete: d, Quit: q");
        enable_raw_mode().unwrap();

        loop {
            if let Event::Key(key) = read_event().unwrap() {
                if let KeyCode::Char(ch) = key.code {
                    match ch {
                        'a' => return CharacterMenuAction::Add,
                        'e' => {
                            disable_raw_mode().unwrap();
                            let mut input = String::new();
                            loop {
                                if let Event::Key(key) = read_event().unwrap() {
                                    if let KeyCode::Enter = key.code {
                                        break;
                                    }
                                    if let KeyCode::Char(ch) = key.code {
                                        input.push(ch);
                                    }
                                }
                            }
                            return CharacterMenuAction::Edit(input.parse::<i32>().unwrap());
                        }
                        'd' => {
                            disable_raw_mode().unwrap();
                            let mut input = String::new();
                            loop {
                                if let Event::Key(key) = read_event().unwrap() {
                                    if let KeyCode::Enter = key.code {
                                        break;
                                    }
                                    if let KeyCode::Char(ch) = key.code {
                                        input.push(ch);
                                    }
                                }
                            }
                            return CharacterMenuAction::Delete(input.parse::<i32>().unwrap());
                        }
                        'q' => return CharacterMenuAction::Quit,
                        _ => (),
                    }
                }
            }
        }
    }

    pub fn edit_player(&mut self, player: Option<Player>) -> Player {
        disable_raw_mode().unwrap();
        fn get_text(_term: &mut Term, old_value: String, stat_name: &str) -> String {
            if !old_value.is_empty() {
                println!("Old {}: {}. Press enter to skip", stat_name, old_value);
            }
            print!("{}: ", stat_name);
            std::io::stdout().flush().unwrap();
            let input = Tui::get_input().trim().to_string();
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
                std::io::stdout().flush().unwrap();
                let input = Tui::get_input().trim().to_string();
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
            std::io::stdout().flush().unwrap();
            //std::io::stdout().flush();
            read_event().unwrap();
        }

        //let mut player: Player = Default::default();
        let mut player: Player = player.unwrap_or_default();

        player.name = get_text(&mut self.term, player.name, "Name");
        player.class = get_text(&mut self.term, player.class, "Class");

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
                std::io::stdout().flush().unwrap();
                let answer = Tui::get_input();
                match answer.trim() {
                    "y" | "yes" => skill.available_after = 0,
                    _ => (),
                }
            }
        }

        print!("Add new skills? ");
        std::io::stdout().flush().unwrap();
        let answer = Tui::get_input();
        match answer.trim() {
            "y" | "yes" => loop {
                print!("Skill name (enter \"q\" to quit): ");
                std::io::stdout().flush().unwrap();
                let name = Tui::get_input().trim().to_string();
                if name == "q" {
                    break;
                }
                print!("Skill cooldown: ");
                std::io::stdout().flush().unwrap();
                let cd = loop {
                    match Tui::get_input().trim().parse::<u32>() {
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
    }
}
