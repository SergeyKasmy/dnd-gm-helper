use crate::status::StatusCooldownType;

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

pub enum EditorAction {
	View(EditorActionViewMode),
	Edit(EditorActionEditMode),
}

pub enum EditorActionViewMode {
	Next,
	Prev,
	Add,
	Edit,
	Delete,
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
