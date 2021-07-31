pub enum MainMenuAction {
    Play,
    Edit,
    Quit,
}

pub enum GameAction {
    UseSkill,
    AddStatus,
    DrainStatusAttacking,
    DrainStatusAttacked,
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
    Editing { buffer: String, field_offset: Option<i8>},
    Delete(usize),
    Quit,
}
