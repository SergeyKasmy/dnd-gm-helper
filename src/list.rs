use crate::id::Id;
use crate::id::OrderNum;
use crate::id::Uid;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct IdList<T>
where
	T: Id, //where T: Id + Serialize + Deserialize
{
	// Not intended for end-user use, only for custom impl IdList<T>
	pub list: IndexMap<Uid, T>,
}

impl<T> IdList<T>
where
	T: Id,
{
	pub fn new(list: IndexMap<Uid, T>) -> Self {
		Self { list }
	}

	pub fn get(&self, id: Uid) -> Option<&T> {
		// TODO: is it safe not to sort it after it may have been possible that the user changed
		// the sorting property with get_map_mut()
		//self.sort();
		self.list.get(&id)
	}

	pub fn get_mut(&mut self, id: Uid) -> Option<&mut T> {
		//self.sort();
		self.list.get_mut(&id)
	}

	pub fn get_by_index(&self, num: OrderNum) -> Option<(&Uid, &T)> {
		self.list.get_index(*num)
	}

	pub fn get_index_of(&self, id: Uid) -> Option<OrderNum> {
		self.list.get_index_of(&id).map(|x| OrderNum(x))
	}

	pub fn iter(&self) -> impl Iterator<Item = (&Uid, &T)> {
		self.list.iter()
	}

	pub fn push(&mut self, new_val: T) -> Uid {
		let biggest_id = if let Some(num) = self.list.keys().max() {
			*num + 1.into()
		} else {
			0.into()
		};

		self.insert(biggest_id, new_val);
		//self.sort();
		biggest_id
	}

	pub fn insert(&mut self, id: Uid, mut new_val: T) {
		*new_val.id() = Some(id);
		self.list.insert(id, new_val);
		//self.sort();
	}

	pub fn remove(&mut self, id: Uid) -> Option<(Uid, T)> {
		let removed = self.list.remove_entry(&id);
		//self.sort();
		removed
	}

	pub fn clear(&mut self) {
		self.list.clear();
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn is_empty(&self) -> bool {
		self.list.is_empty()
	}
}

impl<T: Id> Default for IdList<T> {
	fn default() -> Self {
		Self::new(IndexMap::new())
	}
}
