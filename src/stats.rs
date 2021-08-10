use crate::entity_list::EntityList;
use crate::id::Uid;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct StatList {
	// TODO: uncomment all serde(flatten) attrs when I figure out how to get rid of "invalid type: string "", expected usize" error
	//#[serde(flatten)]
	map: IndexMap<Uid, String>,
}

impl StatList {
	pub fn get_name(&self, id: Uid) -> Option<&str> {
		Some(self.map.get(&id)?.as_str())
	}

	pub fn contains(&self, id: Uid) -> bool {
		self.map.contains_key(&id)
	}
}

impl EntityList for StatList {
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
		self.map.sort_by(|_, a, _, b| a.cmp(&b));
	}
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Stats {
	//#[serde(flatten)]
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
