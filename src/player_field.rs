use crate::STAT_LIST;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PlayerField {
	Name,
	Stat(usize),
	SkillName(usize),
	SkillCD(usize),
}

impl PlayerField {
	pub fn next(&self) -> PlayerField {
		match self {
			PlayerField::Name => {
				let stat_list = STAT_LIST.lock().unwrap();
				let vec = stat_list.as_vec();
				// TODO: avoid using vec, use map's iter directly
				if !vec.is_empty() {
					PlayerField::Stat(0)
				} else {
					// TODO: do the same check as above
					PlayerField::SkillName(0)
				}
			}
			PlayerField::Stat(selected) => {
				if *selected < STAT_LIST.lock().unwrap().len() - 1 {
					PlayerField::Stat(*selected + 1)
				} else {
					PlayerField::SkillName(0)
				}
			}
			PlayerField::SkillName(i) => PlayerField::SkillCD(*i),
			PlayerField::SkillCD(i) => PlayerField::SkillName(*i + 1),
		}
	}

	pub fn prev(&self) -> PlayerField {
		match self {
			PlayerField::Name => PlayerField::Name,
			PlayerField::Stat(i) => {
				if *i == 0 {
					PlayerField::Name
				} else {
					PlayerField::Stat(*i - 1)
				}
			}
			PlayerField::SkillName(i) => {
				if *i == 0 {
					PlayerField::Stat(STAT_LIST.lock().unwrap().len() - 1)
				} else {
					PlayerField::SkillCD(*i - 1)
				}
			}
			PlayerField::SkillCD(i) => PlayerField::SkillName(*i),
		}
	}
}

impl fmt::Display for PlayerField {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		// TODO: avoid holding the Mutex for so long for no reason
		let stat_list = STAT_LIST.lock().unwrap();
		let out = match self {
			PlayerField::Name => "Name",
			PlayerField::Stat(i) => stat_list.get_name(*i),
			PlayerField::SkillName(_) => "", // not intended for actual use
			PlayerField::SkillCD(_) => "",
		};
		write!(formatter, "{}", out)
	}
}
