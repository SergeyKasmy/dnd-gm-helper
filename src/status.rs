use crate::entity_list::EntityList;
use crate::id::Uid;
use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatusList {
	//#[serde(flatten)]
	map: IndexMap<Uid, String>,
}

impl EntityList for StatusList {
	type Entity = String;

	fn new(map: IndexMap<Uid, Self::Entity>) -> Self {
		Self { map }
	}

	fn get_map(&self) -> &IndexMap<Uid, Self::Entity> {
		&self.map
	}

	fn get_map_mut(&mut self) -> &mut IndexMap<Uid, Self::Entity> {
		&mut self.map
	}

	fn sort(&mut self) {
		self.map.sort_by(|_, a, _, b| a.cmp(b));
	}
}

impl Default for StatusList {
	fn default() -> Self {
		Self::new(IndexMap::new())
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
	pub status_type: Uid,
	pub status_cooldown_type: StatusCooldownType,
	pub duration_left: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Statuses {
	//#[serde(flatten)]
	map: IndexMap<Uid, Status>,
}

impl Statuses {
	pub fn drain_by_type(&mut self, status_type: StatusCooldownType) {
		// decrease all statuses duration with the status cooldown type provided
		self.map.iter_mut().for_each(|(_, status)| {
			if status.status_cooldown_type == status_type && status.duration_left > 0 {
				log::debug!("Drained {:?}", status.status_type);
				status.duration_left -= 1
			}
		});
		// remove all statuses that have run out = retain all statuses that haven't yet run out
		self.map.retain(|_, status| status.duration_left > 0);
		self.sort();
	}

	// TODO: combine with the one from the above
	pub fn drain_by_id(&mut self, id: Uid) -> Result<()> {
		let curr = self
			.get_mut(id)
			.ok_or(anyhow::Error::msg("Couldn't find player"))?;
		if curr.duration_left > 0 {
			log::debug!("Drained {:?}, uid {}", curr.status_type, id);
			curr.duration_left -= 1;
		}

		self.map.retain(|_, status| status.duration_left > 0);
		Ok(())
	}
}

impl EntityList for Statuses {
	type Entity = Status;

	fn new(map: IndexMap<Uid, Self::Entity>) -> Self {
		Self { map }
	}

	fn get_map(&self) -> &IndexMap<Uid, Self::Entity> {
		&self.map
	}

	fn get_map_mut(&mut self) -> &mut IndexMap<Uid, Self::Entity> {
		&mut self.map
	}

	fn sort(&mut self) {
		self.map
			.sort_by(|_, a, _, b| a.status_type.to_string().cmp(&b.status_type.to_string()));
	}
}

impl Default for Statuses {
	fn default() -> Self {
		Self::new(IndexMap::new())
	}
}
