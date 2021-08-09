use crate::entity_list::EntityList;
use crate::id::Uid;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

// TODO: use HashMap
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum StatusType {
	Discharge,
	FireAttack,
	FireShield,
	IceShield,
	Blizzard,
	Fusion,
	Luck,

	Knockdown,
	Poison,
	Stun,
}

impl fmt::Display for StatusType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum StatusCooldownType {
	Normal,
	OnAttacking,
	OnGettingAttacked,
	Manual,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Status {
	pub status_type: StatusType,
	pub status_cooldown_type: StatusCooldownType,
	pub duration: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Statuses {
	#[serde(flatten)]
	map: HashMap<Uid, Status>,

	#[serde(skip)]
	sorted_ids: RefCell<Option<Vec<Uid>>>,
}

impl Statuses {
	pub fn drain_by_type(&mut self, status_type: StatusCooldownType) {
		self.invalidate_sorted_ids();
		// decrease all statuses duration with the status cooldown type provided
		self.map.iter_mut().for_each(|(_, status)| {
			if status.status_cooldown_type == status_type && status.duration > 0 {
				log::debug!("Drained {:?}", status.status_type);
				status.duration -= 1
			}
		});
		// remove all statuses that have run out = retain all statuses that haven't yet run out
		self.map.retain(|_, status| status.duration > 0);
	}

	// TODO: combine with the one from the above
	pub fn drain_by_id(&mut self, id: Uid) -> Result<()> {
		let curr = self
			.get_mut(id)
			.ok_or(anyhow::Error::msg("Couldn't find player"))?;
		if curr.duration > 0 {
			log::debug!("Drained {:?}, uid {}", curr.status_type, id);
			curr.duration -= 1;
		}

		self.map.retain(|_, status| status.duration > 0);
		Ok(())
	}
}

impl EntityList for Statuses {
	type Entity = Status;

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
			log::debug!("Sorting status list");
			*self.sorted_ids.borrow_mut() = Some({
				let mut unsorted: Vec<Uid> = self.map.iter().map(|(id, _)| *id).collect();
				unsorted.sort_by(|a, b| {
					self.map
						.get(&a)
						.unwrap()
						.status_type
						.to_string()
						.cmp(&self.map.get(&b).unwrap().status_type.to_string())
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

impl Default for Statuses {
	fn default() -> Self {
		Self::new(HashMap::new())
	}
}
