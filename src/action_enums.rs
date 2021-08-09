use crate::status::StatusCooldownType;
use crate::id::{OrderNum, Uid};

pub enum MainMenuAction {
	Play,
	Edit,
	ReorderPlayers,
	Quit,
}

pub enum GameAction {
	UseSkill,
	AddStatus,
	DrainStatus(StatusCooldownType),

	#[allow(dead_code)]
	ManageMoney,
	ClearStatuses,
	ResetSkillsCD,
	MakeTurn,
	SkipTurn,
	NextPlayerPick,
	Quit,
}

pub enum CharacterMenuAction {
	Add,
	Edit(Uid),
	Editing {
		buffer: String,
		field_offset: Option<i8>,
	},
	DoneEditing,
	Delete(Uid),
	Quit,
}
