use crate::{id::Uid, player::Players, stats::StatList, status::StatusList};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GameState {
	pub players: Players,
	pub order: Vec<Uid>,
	pub stat_list: StatList,
	pub status_list: StatusList,
}
