use crate::entity_list::EntityList;
use crate::id::Uid;
use crate::skill::Skill;
use crate::stats::Stats;
use crate::status::Status;
use crate::status::StatusCooldownType;
use crate::status::Statuses;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

pub type Hp = u16;

pub enum PlayerState {
	Alive(Hp),
	Dead,
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Player {
	// permanent state
	pub name: String,
	pub stats: Stats,
	max_hp: Hp,

	// temporary state
	hp: Hp,
	money: i64,
	pub skills: Vec<Skill>,
	pub statuses: Statuses,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Players {
	//#[serde(flatten)]
	map: HashMap<Uid, Player>,

	#[serde(skip)]
	sorted_ids: RefCell<Option<Vec<Uid>>>,
}

impl EntityList for Players {
	type Entity = Player;

	fn new(map: HashMap<Uid, Self::Entity>) -> Self {
		Self {
			map,
			sorted_ids: RefCell::new(None),
		}
	}

	fn get_map(&self) -> &HashMap<Uid, Self::Entity> {
		&self.map
	}

	fn get_map_mut(&mut self) -> &mut HashMap<Uid, Self::Entity> {
		&mut self.map
	}

	fn sort_ids(&self) -> Vec<Uid> {
		if self.sorted_ids.borrow().is_none() {
			log::debug!("Sorting player list");
			*self.sorted_ids.borrow_mut() = Some({
				let mut unsorted: Vec<Uid> = self.map.iter().map(|(id, _)| *id).collect();
				unsorted.sort_by(|a, b| {
					self.map
						.get(&a)
						.unwrap()
						.name
						.cmp(&self.map.get(&b).unwrap().name)
				});
				unsorted
			});
		}
		match &*self.sorted_ids.borrow() {
			Some(ids) => ids.clone(),
			None => {
				log::error!("Somehow the sorted list of player ids is None even though we should've just created it");
				unreachable!();
			}
		}
	}

	fn invalidate_sorted_ids(&self) {
		*self.sorted_ids.borrow_mut() = None;
	}
}

// TODO: maybe move this impls to a macro
impl Default for Players {
	fn default() -> Self {
		Players {
			map: HashMap::new(),
			sorted_ids: RefCell::new(None),
		}
	}
}
