pub mod term;
pub mod ui_type;

use anyhow::Result;
use dnd_gm_helper::{
	action_enums::{
		EditorAction, EditorActionViewMode, GameAction, MainMenuAction, SettingsAction,
	},
	id::{OrderNum, Uid},
	list::SetList,
	player::{Player, Players},
	side_effect::SideEffect,
	skill::Skill,
	stats::StatList,
	status::{Status, StatusList},
};
// TODO: get rid of this
use tui::{layout::Rect, widgets::Table};

#[derive(Clone)]
pub enum EditorMode {
	View {
		selected: Option<OrderNum>,
	},
	Edit {
		selected: OrderNum,
		error: Option<String>,
	},
}

pub trait Ui {
	fn draw_menu(
		&self,
		items: &[impl AsRef<str>],
		statusbar_text: impl AsRef<str>,
	) -> Result<Option<usize>>;
	fn draw_main_menu(&self) -> Result<MainMenuAction>;
	fn draw_settings_menu(&self) -> Result<SettingsAction>;
	fn draw_game(&self, player: &Player, stat_list: &StatList) -> Result<GameAction>;
	fn choose_skill(&self, skills: &[Skill]) -> Result<Option<OrderNum>>;
	fn choose_status(&self, status_list: &StatusList) -> Result<Option<Status>>;
	fn get_money_amount(&self) -> Result<i64>;
	fn pick_player<'a>(
		&self,
		players: &'a Players,
		ignore: Option<Uid>,
	) -> Result<Option<&'a Player>>;

	fn draw_editor<'a, F>(
		&self,
		mode: EditorMode,
		list_title: Option<impl AsRef<str>>,
		list_items: &[impl AsRef<str>],
		details: Option<F>,
	) -> Result<EditorAction>
	where
		// TODO: Use F: Fn(Rect) -> Vec<(Box<dyn Widget>, Rect)>,
		F: Fn(Rect) -> Vec<(Table<'a>, Rect)>;

	fn draw_character_menu(
		&self,
		players: &Players,
		stat_list: &StatList,
	) -> Result<EditorActionViewMode>;

	fn draw_setlist(&self, setlist: &SetList<String>) -> Result<EditorActionViewMode>;
	fn edit_player(
		&self,
		players: &Players,
		id: Uid,
		stat_list: &StatList,
		status_list: &StatusList,
	) -> Result<Option<Player>>;

	fn edit_setlist(
		&self,
		list: &SetList<String>,
		item: String,
		item_ordernum: OrderNum,
		title: Option<impl AsRef<str>>,
	) -> Result<String>;

	fn edit_side_effect(
		&self,
		old_side_effect: Option<SideEffect>,
		status_list: &StatusList,
	) -> Result<Option<SideEffect>>;

	fn reorder_players(&self, old_player_order: &[Uid], players: &mut Players) -> Result<Vec<Uid>>;
}
