use crate::id::OrderNum;
use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StatList {
	list: IndexSet<String>,
}

impl StatList {
	pub fn contains(&self, name: &str) -> bool {
		self.list.contains(name)
	}

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

	pub fn is_empty(&self) -> bool {
		self.list.is_empty()
	}

	pub fn sort(&mut self) {
		self.list.sort_by(|a, b| a.cmp(b))
	}
}

/*
impl EntityList for StatList {
	impl_default_entitylist!(StatDef);
	fn sort(&mut self) {
		self.map.sort_by(|_, a, _, b| a.name.cmp(&b.name));
	}
}
*/

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
