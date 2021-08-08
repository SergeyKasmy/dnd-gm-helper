pub enum MainMenuAction {
	Play,
	Edit,
	ReorderPlayers,
	Quit,
}

pub enum GameAction {
	UseSkill,
	AddStatus,
	DrainStatusAttacking,
	DrainStatusAttacked,

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
	Edit(usize),
	Editing {
		buffer: String,
		field_offset: Option<i8>,
	},
	DoneEditing,
	Delete(usize),
	Quit,
}
