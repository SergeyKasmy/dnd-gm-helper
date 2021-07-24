use crossterm::event::{read as read_event, Event, KeyCode};
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    widgets::{List, ListItem},
    Terminal,
};

pub struct Tui {
    term: Terminal<CrosstermBackend<Stdout>>,
}

#[derive(Debug)]
pub enum MainMenuButton {
    Play,
    Edit,
    Quit,
}

impl Tui {
    pub fn new() -> Tui {
        //crossterm::terminal::enable_raw_mode().unwrap();

        Tui {
            term: Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap(),
        }
    }

    pub fn draw_main_menu(&mut self) -> MainMenuButton {
        self.term.clear().unwrap();
        self.term
            .draw(|frame| {
                let items = [
                    ListItem::new(format!("{:?}", MainMenuButton::Play)),
                    ListItem::new(format!("{:?}", MainMenuButton::Edit)),
                    ListItem::new(format!("{:?}", MainMenuButton::Quit)),
                ];
                let list = List::new(items);
                frame.render_widget(list, frame.size());
            })
            .unwrap();

        loop {
            if let Event::Key(key) = read_event().unwrap() {
                if let KeyCode::Char(ch) = key.code {
                    match ch {
                        '1' => return MainMenuButton::Play,
                        '2' => return MainMenuButton::Edit,
                        '3' | 'q' => return MainMenuButton::Quit,
                        _ => (),
                    }
                }
            }
        }
    }
}
