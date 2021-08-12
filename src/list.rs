use crate::id::Id;
use crate::id::OrderNum;
use crate::id::Uid;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetList<T>
where
	T: Eq + Hash /* for the hashing itself */ + Ord /* for sorting */ + Display, /* for getting names/pretty list */
{
	list: IndexSet<T>,
}

impl<T> SetList<T>
where
	T: Eq + Hash + Ord + Display,
{
	pub fn new(list: IndexSet<T>) -> Self {
		Self { list }
	}

	pub fn contains<Q>(&self, val: &Q) -> bool
	where
		T: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		self.list.contains(val)
	}

	pub fn iter(&self) -> impl Iterator<Item = &T> {
		self.list.iter()
	}

	pub fn get(&self, num: OrderNum) -> Option<&T> {
		self.list.get_index(*num)
	}

	pub fn get_index<Q>(&self, val: &Q) -> Option<OrderNum>
	where
		T: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		self.list.get_full(val).map(|(id, _)| OrderNum(id))
	}

	// TODO: is there a way not to allocate?
	// There should be a way to check if T: AsRef<str>...
	pub fn get_names(&self) -> Vec<String> {
		self.list.iter().map(|x| x.to_string()).collect()
	}

	pub fn insert(&mut self, val: T) {
		self.list.insert(val);
		self.sort();
	}

	pub fn remove<Q>(&mut self, val: &Q) -> Option<(OrderNum, T)>
	where
		T: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		self.list
			.shift_remove_full(val)
			.map(|(num, x)| (num.into(), x))
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

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct IdList<T>
where
	T: Id + Ord, //where T: Id + Serialize + Deserialize
{
	// Not intended for end-user use, only for custom impl IdList<T>
	pub list: IndexMap<Uid, T>,
}

impl<T> IdList<T>
where
	T: Id + Ord,
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
		self.sort();
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
		self.sort();
		biggest_id
	}

	pub fn insert(&mut self, id: Uid, mut new_val: T) {
		*new_val.id() = Some(id);
		self.list.insert(id, new_val);
		self.sort();
	}

	pub fn remove(&mut self, id: Uid) -> Option<(Uid, T)> {
		let removed = self.list.remove_entry(&id);
		self.sort();
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

	pub fn sort(&mut self) {
		self.list.sort_by(|_, a, _, b| a.cmp(b))
	}
}

impl<T: Id + Ord> Default for IdList<T> {
	fn default() -> Self {
		Self::new(IndexMap::new())
	}
}
