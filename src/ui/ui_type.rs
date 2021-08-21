use super::{term::Term, Ui};

use anyhow::Result;
use dnd_gm_helper::{
	action_enums::{EditorActionViewMode, MainMenuAction, SettingsAction},
	id::{OrderNum, Uid},
	list::SetList,
	player::{Player, Players},
	side_effect::SideEffect,
	skill::Skill,
	stats::StatList,
	status::{Status, StatusList},
};

pub enum UiType {
	TermTui(Term),
}

impl Ui for UiType {
	fn draw_menu(
		&self,
		items: &[impl AsRef<str>],
		statusbar_text: impl AsRef<str>,
	) -> Result<Option<usize>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.draw_menu(items, statusbar_text),
		}
	}

	fn draw_main_menu(&self) -> Result<MainMenuAction> {
		match &self {
			Self::TermTui(term_tui) => term_tui.draw_main_menu(),
		}
	}

	fn draw_settings_menu(&self) -> Result<SettingsAction> {
		match &self {
			Self::TermTui(term_tui) => term_tui.draw_settings_menu(),
		}
	}

	fn draw_game(
		&self,
		player: &Player,
		stat_list: &StatList,
	) -> Result<dnd_gm_helper::action_enums::GameAction> {
		match &self {
			Self::TermTui(term_tui) => term_tui.draw_game(player, stat_list),
		}
	}

	fn choose_skill(&self, skills: &[Skill]) -> Result<Option<dnd_gm_helper::id::OrderNum>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.choose_skill(skills),
		}
	}

	fn choose_status(&self, status_list: &StatusList) -> Result<Option<Status>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.choose_status(status_list),
		}
	}

	fn get_money_amount(&self) -> Result<i64> {
		match &self {
			Self::TermTui(term_tui) => term_tui.get_money_amount(),
		}
	}

	fn pick_player<'a>(
		&self,
		players: &'a Players,
		ignore: Option<Uid>,
	) -> Result<Option<&'a Player>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.pick_player(players, ignore),
		}
	}

	fn draw_character_menu(
		&self,
		players: &Players,
		stat_list: &StatList,
	) -> Result<EditorActionViewMode> {
		match &self {
			Self::TermTui(term_tui) => term_tui.draw_character_menu(players, stat_list),
		}
	}

	fn draw_setlist(&self, setlist: &SetList<String>) -> Result<EditorActionViewMode> {
		match &self {
			Self::TermTui(term_tui) => term_tui.draw_setlist(setlist),
		}
	}

	fn edit_player(
		&self,
		players: &Players,
		id: Uid,
		stat_list: &StatList,
		status_list: &StatusList,
	) -> Result<Option<Player>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.edit_player(players, id, stat_list, status_list),
		}
	}

	fn edit_setlist(
		&self,
		list: &SetList<String>,
		item: String,
		item_ordernum: OrderNum,
		title: Option<impl AsRef<str>>,
	) -> Result<String> {
		match &self {
			Self::TermTui(term_tui) => term_tui.edit_setlist(list, item, item_ordernum, title),
		}
	}

	fn edit_side_effect(
		&self,
		old_side_effect: Option<SideEffect>,
		status_list: &StatusList,
	) -> Result<Option<SideEffect>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.edit_side_effect(old_side_effect, status_list),
		}
	}

	fn reorder_players(&self, old_player_order: &[Uid], players: &mut Players) -> Result<Vec<Uid>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.reorder_players(old_player_order, players),
		}
	}

	fn messagebox_with_options(
		&self,
		desc: impl AsRef<str>,
		options: &[impl AsRef<str>],
		is_vertical: bool,
	) -> Result<Option<OrderNum>> {
		match &self {
			Self::TermTui(term_tui) => term_tui.messagebox_with_options(desc, options, is_vertical),
		}
	}

	fn messagebox_with_input_field(&self, desc: impl AsRef<str>) -> Result<String> {
		match &self {
			Self::TermTui(term_tui) => term_tui.messagebox_with_input_field(desc),
		}
	}

	fn messagebox_yn(&self, desc: impl AsRef<str>) -> Result<bool> {
		match &self {
			Self::TermTui(term_tui) => term_tui.messagebox_yn(desc),
		}
	}

	fn messagebox(&self, desc: impl AsRef<str>) -> Result<()> {
		match &self {
			Self::TermTui(term_tui) => term_tui.messagebox(desc),
		}
	}
}
