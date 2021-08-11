use crate::entity::{Entity, EntityList};
use crate::id::Uid;
use crate::impl_default_entitylist;
use crate::impl_entity;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct StatDef {
	id: Option<Uid>,
	pub name: String,
}
impl_entity!(StatDef);

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StatList {
	map: IndexMap<Uid, StatDef>,
}

impl StatList {
	pub fn get_name(&self, id: Uid) -> Option<&str> {
		Some(self.map.get(&id)?.name.as_str())
	}

	pub fn contains(&self, id: Uid) -> bool {
		self.map.contains_key(&id)
	}
}

impl EntityList for StatList {
	impl_default_entitylist!(StatDef);
	fn sort(&mut self) {
		self.map.sort_by(|_, a, _, b| a.name.cmp(&b.name));
	}
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Stats {
	map: IndexMap<Uid, i32>,
}

impl Stats {
	pub fn new(mut map: IndexMap<Uid, i32>, stat_list: &StatList) -> Stats {
		// ignore all stats that's id doesn't exist
		map.retain(|&id, _| stat_list.contains(id));
		Stats { map }
	}

	pub fn get(&self, id: Uid) -> i32 {
		*self.map.get(&id).unwrap_or(&0)
	}

	/*
	pub fn get_mut(&mut self, id: usize) -> &mut i32 {
		// TODO: check if id is a real stat
		if !self.map.contains_key(&id) {
			self.map.insert(id, 0);
		}

		self.map.get_mut(&id).unwrap()
	}
	*/

	pub fn set(&mut self, id: Uid, new_val: i32) {
		if new_val == 0 {
			self.map.remove(&id);
		} else {
			self.map.insert(id, new_val);
		}
	}
}
