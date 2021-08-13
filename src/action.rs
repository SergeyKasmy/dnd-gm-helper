use crate::id::{OrderNum, Uid};
use crate::status::StatusCooldownType;

pub enum ClientAction {
	ServerMessage(ServerMessage),
	//UserInteraction,
}

pub enum ServerMessage {
	GameList(Vec<String>),
	PlayerData(PlayerData),
}

pub enum ServerAction {
	// client Uid
	ClientMessage(Uid, ClientMessage),
}

pub enum ClientMessage {
	RequestGameList,
	RequestPlayerData(PlayerData),
	// game name
	AddNewGame(String),
	SetCurrentGame(OrderNum),
	Save,
}

pub enum PlayerData {
	IsEmpty(Option<bool>),
}

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
