use crate::list::SetList;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type StatList = SetList<String>;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Stats {
	map: IndexMap<String, i32>,
}

impl Stats {
	pub fn new(mut map: IndexMap<String, i32>, stat_list: &StatList) -> Stats {
		// ignore all stats that don't exist
		map.retain(|name, _| stat_list.contains(name));
		Stats { map }
	}

	pub fn get(&self, name: &str) -> i32 {
		*self.map.get(name).unwrap_or(&0)
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

	pub fn set<T: AsRef<str>>(&mut self, name: T, new_val: i32) {
		if new_val == 0 {
			self.map.remove(name.as_ref());
		} else {
			self.map.insert(name.as_ref().to_string(), new_val);
		}
	}
}
