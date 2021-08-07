use tui::widgets::ListState;

pub trait ListStateExt {
    fn next(&mut self, len: usize) -> Option<usize>;
    fn prev(&mut self, len: usize) -> Option<usize>;
}

impl ListStateExt for ListState {
    fn next(&mut self, len: usize) -> Option<usize> {
        let next_num = match self.selected() {
            Some(num) if len > 0 => {
                if num >= len - 1 {
                    Some(0)
                } else {
                    Some(num + 1)
                }
            }
            None if len > 0 => Some(0),
            _ => None,
        };

        self.select(next_num);
        next_num
    }

    fn prev(&mut self, len: usize) -> Option<usize> {
        let prev_num = match self.selected() {
            Some(num) if len > 0 => {
                if num == 0 {
                    Some(len - 1)
                } else {
                    Some(num - 1)
                }
            }
            None if len > 0 => Some(0),
            _ => None,
        };

        self.select(prev_num);
        prev_num
    }
}
