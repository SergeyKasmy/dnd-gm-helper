use crate::entity::{Entity, EntityList};
use crate::id::Uid;
use crate::impl_default_entitylist;
use crate::impl_entity;
use anyhow::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct StatusDef {
	id: Option<Uid>,
	pub name: String,
}
impl_entity!(StatusDef);

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct StatusList {
	map: IndexMap<Uid, StatusDef>,
}

impl EntityList for StatusList {
	impl_default_entitylist!(StatusDef);
	fn sort(&mut self) {
		self.map.sort_by(|_, a, _, b| a.name.cmp(&b.name));
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
	id: Option<Uid>,
	pub status_type: Uid,
	pub status_cooldown_type: StatusCooldownType,
	pub duration_left: u32,
}
impl_entity!(Status);

impl Status {
	pub fn new(status_type: Uid, status_cooldown_type: StatusCooldownType, duration: u32) -> Self {
		Self {
			id: None,
			status_type,
			status_cooldown_type,
			duration_left: duration,
		}
	}
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct Statuses {
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
	impl_default_entitylist!(Status);
	fn sort(&mut self) {
		self.map
			.sort_by(|_, a, _, b| a.status_type.to_string().cmp(&b.status_type.to_string()));
	}
}
