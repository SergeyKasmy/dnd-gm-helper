use crate::id::Uid;
use crate::impl_id_trait;
use crate::list::IdList;
use crate::list::SetList;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub type StatusList = SetList<String>;

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
impl_id_trait!(Status);

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

pub type Statuses = IdList<Status>;

impl Statuses {
	pub fn drain_by_type(&mut self, status_type: StatusCooldownType) {
		// decrease all statuses duration with the status cooldown type provided
		self.list.iter_mut().for_each(|(_, status)| {
			if status.status_cooldown_type == status_type && status.duration_left > 0 {
				log::debug!("Drained {:?}", status.status_type);
				status.duration_left -= 1
			}
		});
		// remove all statuses that have run out = retain all statuses that haven't yet run out
		self.list.retain(|_, status| status.duration_left > 0);
		//self.sort();
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

		self.list.retain(|_, status| status.duration_left > 0);
		Ok(())
	}
}

/*
impl IdList for Statuses {
	fn sort(&mut self) {
		self.list
			.sort_by(|_, a, _, b| a.status_type.to_string().cmp(&b.status_type.to_string()));
	}
}
*/
