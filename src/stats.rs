use crate::STAT_LIST;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatList {
	map: HashMap<usize, String>,
	// TODO: maybe keep a sorted vec inside for faster access to ui stuff
	// and selection -> id convertion
}

impl StatList {
	pub fn new(map: HashMap<usize, String>) -> Self {
		StatList { map }
	}

	pub fn len(&self) -> usize {
		self.map.len()
	}

	pub fn get_name(&self, id: usize) -> &str {
		// TODO: check if exists. May crash otherwise
		self.map.get(&id).unwrap().as_str()
	}

	pub fn contains(&self, id: usize) -> bool {
		self.map.contains_key(&id)
	}

	pub fn as_vec(&self) -> Vec<(&usize, &str)> {
		let mut vec = self
			.map
			.iter()
			.map(|(id, name)| (id, name.as_str()))
			.collect::<Vec<(&usize, &str)>>();
		vec.sort_by(|a, b| a.1.cmp(b.1));
		vec
	}
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Stats {
	map: HashMap<usize, i32>,
}

impl Stats {
	pub fn new(mut map: HashMap<usize, i32>) -> Stats {
		// ignore all stats that's id doesn't exist
		map.retain(|&id, _| STAT_LIST.lock().unwrap().contains(id));
		Stats { map }
	}

	pub fn get(&self, id: usize) -> i32 {
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

	pub fn set(&mut self, id: usize, new_val: i32) {
		if new_val == 0 {
			self.map.remove(&id);
		} else {
			self.map.insert(id, new_val);
		}
	}
}
