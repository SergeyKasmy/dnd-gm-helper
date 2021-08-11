use crate::entity::{Entity, EntityList};
use crate::id::OrderNum;
use crate::id::Uid;
use crate::impl_default_entitylist;
use crate::impl_entity;
use anyhow::Result;
use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(transparent)]
pub struct StatusList {
	list: IndexSet<String>,
}

/*
impl EntityList for StatusList {
	impl_default_entitylist!(StatusDef);
	fn sort(&mut self) {
		self.map.sort_by(|_, a, _, b| a.cmp(&b));
	}
}
*/

impl StatusList {
	pub fn iter(&self) -> impl Iterator<Item = &String> {
		self.list.iter()
	}

	// TODO: maybe there's a better way?
	pub fn get(&self, num: OrderNum) -> Option<&str> {
		self.get_names().get(*num).map(|x| *x)
	}

	pub fn get_names(&self) -> Vec<&str> {
		self.list.iter().map(|x| x.as_str()).collect()
	}

	pub fn get_index(&self, name: &str) -> Option<OrderNum> {
		self.list.get_full(name).map(|(id, _)| OrderNum(id))
	}

	pub fn insert(&mut self, name: String) {
		self.list.insert(name);
		self.sort();
	}

	pub fn remove<T: AsRef<str>>(&mut self, name: T) -> Option<String> {
		self.list.shift_remove_full(name.as_ref()).map(|(_, s)| s)
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn sort(&mut self) {
		self.list.sort_by(|a, b| a.cmp(b))
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
	pub status_type: String,
	pub status_cooldown_type: StatusCooldownType,
	pub duration_left: u32,
}
impl_entity!(Status);

impl Status {
	pub fn new(
		status_type: String,
		status_cooldown_type: StatusCooldownType,
		duration: u32,
	) -> Self {
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
