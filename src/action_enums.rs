use crate::{id::OrderNum, status::StatusCooldownType};

pub enum MainMenuAction {
	Play,
	EditPlayers,
	ReorderPlayers,
	Settings,
	Quit,
}

pub enum SettingsAction {
	EditStats,
	EditStatuses,
	GoBack,
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

pub enum EditorAction {
	View(EditorActionViewMode),
	Edit(EditorActionEditMode),
}

pub enum EditorActionViewMode {
	Next,
	Prev,
	Add,
	Edit(OrderNum),
	Delete(OrderNum),
	Quit,
}

pub enum EditorActionEditMode {
	Char(char),
	Pop,
	Next,
	Prev,
	DoneWithField,
	Done,
}
