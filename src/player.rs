use crate::entity::{Entity, EntityList};
use crate::id::Uid;
use crate::impl_default_entitylist;
use crate::impl_entity;
use crate::skill::Skill;
use crate::stats::Stats;
use crate::status::Status;
use crate::status::StatusCooldownType;
use crate::status::Statuses;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type Hp = u16;

pub enum PlayerState {
	Alive(Hp),
	Dead,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Player {
	// TODO: tmp pub
	pub id: Option<Uid>,
	// permanent state
	pub name: String,
	pub stats: Stats,
	// TODO: tmp pub
	pub max_hp: Hp,

	// temporary state
	// TODO: tmp pub
	pub hp: Hp,
	// TODO: tmp pub
	pub money: i64,
	pub skills: Vec<Skill>,
	pub statuses: Statuses,
}
impl_entity!(Player);

impl Player {
	pub fn turn(&mut self) {
		log::debug!("{}'s turn has ended", self.name);
		self.skills.iter_mut().for_each(|skill| {
			if skill.cooldown_left > 0 {
				skill.cooldown_left -= 1
			}
		});
		self.drain_status_by_type(StatusCooldownType::Normal);
	}

	pub fn add_status(&mut self, status: Status) {
		self.statuses.push(status);
	}

	pub fn drain_status_by_type(&mut self, status_type: StatusCooldownType) {
		log::debug!(
			"Draining statuses for {} with type {:?}",
			self.name,
			status_type
		);
		self.statuses.drain_by_type(status_type);
	}

	fn get_player_state(&self) -> PlayerState {
		if self.hp == 0 {
			PlayerState::Dead
		} else {
			PlayerState::Alive(self.hp)
		}
	}

	pub fn damage(&mut self, amount: Hp) -> PlayerState {
		if let Some(hp) = self.hp.checked_add(amount) {
			self.hp = hp;
		}

		self.get_player_state()
	}

	pub fn heal(&mut self, mut amount: Hp) -> PlayerState {
		if self.hp + amount > self.max_hp {
			amount = self.max_hp - self.hp;
		}

		self.hp += amount;
		self.get_player_state()
	}

	pub fn manage_money(&mut self, diff: i64) -> i64 {
		//let diff = term.get_money_amount()?;
		log::debug!("Adding {} money to Player {}", diff, self.name);
		self.money += diff;
		self.money
	}
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Players {
	map: IndexMap<Uid, Player>,
}

impl EntityList for Players {
	impl_default_entitylist!(Player);
	fn sort(&mut self) {
		self.map.sort_by(|_, a, _, b| a.name.cmp(&b.name));
	}
}
