use crate::entity_list::EntityList;
use crate::id::OrderNum;
use crate::stats::StatList;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PlayerField {
	Name,
	Stat(OrderNum),
	SkillName(OrderNum),
	SkillCD(OrderNum),
}

impl PlayerField {
	pub fn next(&self, stat_list: &StatList) -> Self {
		match self {
			PlayerField::Name => {
				if !stat_list.is_empty() {
					PlayerField::Stat(OrderNum(0))
				} else {
					// TODO: do the same check as above
					PlayerField::SkillName(OrderNum(0))
				}
			}
			PlayerField::Stat(selected) => {
				if *selected < (stat_list.len() - 1).into() {
					PlayerField::Stat(OrderNum(**selected + 1))
				} else {
					PlayerField::SkillName(OrderNum(0))
				}
			}
			PlayerField::SkillName(i) => PlayerField::SkillCD(*i),
			PlayerField::SkillCD(i) => PlayerField::SkillName(OrderNum(**i + 1)),
		}
	}

	pub fn prev(&self, stat_list: &StatList) -> Self {
		match self {
			PlayerField::Name => PlayerField::Name,
			PlayerField::Stat(i) => {
				if **i == 0 {
					PlayerField::Name
				} else {
					PlayerField::Stat(OrderNum(**i - 1))
				}
			}
			PlayerField::SkillName(i) => {
				if **i == 0 {
					PlayerField::Stat(OrderNum(stat_list.len() - 1))
				} else {
					PlayerField::SkillCD(OrderNum(**i - 1))
				}
			}
			PlayerField::SkillCD(i) => PlayerField::SkillName(*i),
		}
	}
}
