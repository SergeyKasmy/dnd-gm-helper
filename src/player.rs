use crate::entity_list::EntityList;
use crate::skill::Skill;
use crate::stats::Stats;
use crate::status::Status;
use crate::status::StatusCooldownType;
use crate::term::Term;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

pub type Hp = u16;

pub enum PlayerState {
	Alive(Hp),
	Dead,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct Player {
	// permanent state
	pub name: String,
	pub stats: Stats,
	max_hp: Hp,

	// temporary state
	hp: Hp,
	money: i64,
	pub skills: Vec<Skill>,
	pub statuses: Vec<Status>,
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
		// decrease all statuses duration with the status cooldown type provided
		self.statuses.iter_mut().for_each(|status| {
			if status.status_cooldown_type == status_type && status.duration > 0 {
				log::debug!("Drained {:?}", status.status_type);
				status.duration -= 1
			}
		});
		// remove all statuses that have run out = retain all statuses that haven't yet run out
		self.statuses.retain(|status| status.duration > 0);
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

	pub fn manage_money(&mut self, term: &Term) {
		let diff = term.get_money_amount();
		log::debug!("Adding {} money to Player {}", diff, self.name);
		self.money += diff;
	}
}

#[derive(Debug)]
pub struct Players {
	map: HashMap<usize, Player>,
	sorted_ids: RefCell<Option<Vec<usize>>>,
}

/*
impl Players {
	pub fn new(new_map: HashMap<usize, Player>) -> Players {
		Players {
			map: new_map.into_iter().map(|(id, pl)| (id, pl)).collect(),
			sorted_ids: RefCell::new(None),
		}
	}

	pub fn get(&self, id: usize) -> Option<&Player> {
		self.map.get(&id)
	}

	pub fn get_mut(&mut self, id: usize) -> Option<&mut Player> {
		*self.sorted_ids.borrow_mut() = None;
		self.map.get_mut(&id)
	}

	pub fn insert(&mut self, id: usize, new_val: Player) {
		*self.sorted_ids.borrow_mut() = None;
		self.map.insert(id, new_val);
	}

	pub fn remove(&mut self, id: usize) -> Option<(usize, Player)> {
		*self.sorted_ids.borrow_mut() = None;
		self.map.remove_entry(&id)
	}

	pub fn keys(&self) -> std::collections::hash_map::Keys<usize, Player> {
		self.map.keys()
	}

	pub fn len(&self) -> usize {
		self.map.len()
	}

	pub fn is_empty(&self) -> bool {
		self.map.is_empty()
	}

	pub fn sorted_ids(&self) -> Vec<usize> {
		if self.sorted_ids.borrow().is_none() {
			log::debug!("Sorting player list");
			*self.sorted_ids.borrow_mut() = Some({
				let mut unsorted: Vec<usize> = self.map.iter().map(|(id, _)| *id).collect();
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
}
*/

impl EntityList for Players {
	type Entity = Player;

	fn new(map: HashMap<usize, Self::Entity>) -> Self {
		Self {
			map,
			sorted_ids: RefCell::new(None),
		}
	}

	fn get_map(&self) -> &HashMap<usize, Self::Entity> {
		&self.map
	}

	fn get_map_mut(&mut self) -> &mut HashMap<usize, Self::Entity> {
		&mut self.map
	}

	fn sort_ids(&self) -> Vec<usize> {
		if self.sorted_ids.borrow().is_none() {
			log::debug!("Sorting player list");
			*self.sorted_ids.borrow_mut() = Some({
				let mut unsorted: Vec<usize> = self.map.iter().map(|(id, _)| *id).collect();
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

impl Default for Players {
	fn default() -> Self {
		Players {
			map: HashMap::new(),
			sorted_ids: RefCell::new(None),
		}
	}
}

impl Serialize for Players {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut smap = serializer.serialize_map(Some(self.map.len()))?;
		for (id, player) in self.map.iter() {
			smap.serialize_entry(id, player)?;
		}
		smap.end()
	}
}

struct PlayersVisitor {
	marker: PhantomData<fn() -> Players>,
}

impl PlayersVisitor {
	fn new() -> Self {
		PlayersVisitor {
			marker: PhantomData,
		}
	}
}

impl<'de> Visitor<'de> for PlayersVisitor {
	type Value = Players;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("StatList.map<usize, String>")
	}

	fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
	where
		M: MapAccess<'de>,
	{
		let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

		while let Some((id, pl)) = access.next_entry()? {
			map.insert(id, pl);
		}

		Ok(Players::new(map))
	}
}

impl<'de> Deserialize<'de> for Players {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_map(PlayersVisitor::new())
	}
}
