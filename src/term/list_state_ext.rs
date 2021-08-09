use crate::id::OrderNum;
use tui::widgets::ListState;

pub trait ListStateExt {
	fn next(&mut self, len: usize) -> Option<usize>;
	fn prev(&mut self, len: usize) -> Option<usize>;
	fn selected_onum(&self) -> Option<OrderNum>;
	fn select_onum(&mut self, num: Option<OrderNum>);
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

	fn selected_onum(&self) -> Option<OrderNum> {
		self.selected().map(Into::into)
	}

	fn select_onum(&mut self, num: Option<OrderNum>) {
		self.select(num.map(|x| *x))
	}
}
