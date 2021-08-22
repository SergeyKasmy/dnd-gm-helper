use std::cell::RefCell;

use super::Ui;

use anyhow::Result;
use cursive::Cursive;
use cursive::CursiveExt;
use cursive::views::{Dialog, SelectView};
use dnd_gm_helper::action_enums::MainMenuAction;
use dnd_gm_helper::id::OrderNum;

pub struct UiCursive {
    cursive: RefCell<Cursive>,
}

impl UiCursive {
    pub fn new() -> Self { Self { cursive: RefCell::new(Cursive::default()) } }
}

impl Ui for UiCursive {
    fn draw_main_menu(&self) -> Result<MainMenuAction> {
        //let items = &["Start game", "Manage characters", "Settings", "Quit"];
        let mut selected: Option<MainMenuAction> = None;
        let select_view = SelectView::<String>::new().item_str("Start game").item_str("Manage characters").item_str("Settings").item_str("Quit").on_submit(|cur: &mut Cursive, item: &str| {
            selected = Some(match item {
                "Start game" => MainMenuAction::Play,
                "Manage characters" => MainMenuAction::EditPlayers,
                "Settings" => MainMenuAction::Settings,
                "Quit" => MainMenuAction::Quit,
            });
            cur.quit();
        });
        self.cursive.borrow_mut().add_layer(select_view);
        self.cursive.borrow_mut().run();

        Ok(selected.take().unwrap())
    }

    fn draw_settings_menu(&self) -> anyhow::Result<dnd_gm_helper::action_enums::SettingsAction> {
        todo!()
    }

    fn draw_game(&self, player: &dnd_gm_helper::player::Player, stat_list: &dnd_gm_helper::stats::StatList) -> anyhow::Result<dnd_gm_helper::action_enums::GameAction> {
        todo!()
    }

    fn choose_skill(&self, skills: &[dnd_gm_helper::skill::Skill]) -> anyhow::Result<Option<dnd_gm_helper::id::OrderNum>> {
        todo!()
    }

    fn choose_status(&self, status_list: &dnd_gm_helper::status::StatusList) -> anyhow::Result<Option<dnd_gm_helper::status::Status>> {
        todo!()
    }

    fn get_money_amount(&self) -> anyhow::Result<i64> {
        todo!()
    }

    fn pick_player<'a>(
		&self,
		players: &'a dnd_gm_helper::player::Players,
		ignore: Option<dnd_gm_helper::id::Uid>,
	) -> anyhow::Result<Option<&'a dnd_gm_helper::player::Player>> {
        todo!()
    }

    fn draw_character_menu(
		&self,
		players: &dnd_gm_helper::player::Players,
		stat_list: &dnd_gm_helper::stats::StatList,
	) -> anyhow::Result<dnd_gm_helper::action_enums::EditorActionViewMode> {
        todo!()
    }

    fn draw_setlist(&self, setlist: &dnd_gm_helper::list::SetList<String>) -> anyhow::Result<dnd_gm_helper::action_enums::EditorActionViewMode> {
        todo!()
    }

    fn edit_player(
		&self,
		players: &dnd_gm_helper::player::Players,
		id: dnd_gm_helper::id::Uid,
		stat_list: &dnd_gm_helper::stats::StatList,
		status_list: &dnd_gm_helper::status::StatusList,
	) -> anyhow::Result<Option<dnd_gm_helper::player::Player>> {
        todo!()
    }

    fn edit_setlist(
		&self,
		list: &dnd_gm_helper::list::SetList<String>,
		item: String,
		item_ordernum: dnd_gm_helper::id::OrderNum,
		title: Option<impl AsRef<str>>,
	) -> anyhow::Result<String> {
        todo!()
    }

    fn edit_side_effect(
		&self,
		old_side_effect: Option<dnd_gm_helper::side_effect::SideEffect>,
		status_list: &dnd_gm_helper::status::StatusList,
	) -> anyhow::Result<Option<dnd_gm_helper::side_effect::SideEffect>> {
        todo!()
    }

    fn reorder_players(&self, old_player_order: &[dnd_gm_helper::id::Uid], players: &mut dnd_gm_helper::player::Players) -> anyhow::Result<Vec<dnd_gm_helper::id::Uid>> {
        todo!()
    }

    fn messagebox_with_options(
		&self,
		desc: impl AsRef<str>,
		options: &[impl AsRef<str>],
		is_vertical: bool,
	) -> anyhow::Result<Option<dnd_gm_helper::id::OrderNum>> {
        // TODO
        Ok(Some(OrderNum(0)))
    }

    fn messagebox_with_input_field(&self, desc: impl AsRef<str>) -> anyhow::Result<String> {
        todo!()
    }

    fn messagebox_yn(&self, desc: impl AsRef<str>) -> anyhow::Result<bool> {
        todo!()
    }

    fn messagebox(&self, desc: impl AsRef<str>) -> anyhow::Result<()> {
        todo!()
    }
}
