use crate::game_state::GameState;
use crate::id::OrderNum;
use anyhow::Result;

pub struct Server {
	// TODO: mb use an IdList instead
	games: Vec<(String, GameState)>,
	// TODO: maybe get a specific GameState out of the server and use it directly instead?
	current_game_num: Option<OrderNum>,
}

impl Server {
	// TODO: maybe allow to specify a specific save file in params?
	// TODO: add enums do indicate that the db is corrupted
	pub fn new() -> Result<Server> {
		let mut games: Vec<(String, GameState)>;
		let file_contents = std::fs::read_to_string("games.json");
		if let Ok(json) =
			file_contents.map_err(|e| log::info!("games.json could not be read: {}", e))
		{
			match serde_json::from_str(&json) {
				Ok(data) => {
					log::debug!("Read from the db: {:#?}", data);
					games = data;
				}
				Err(e) => {
					log::error!("The database is corrupted: {}", e);
					/*
					if term.messagebox_yn("The database is corrupted. Continue?")? {
						let db_bak = format!(
							"games.json.bak-{}",
							std::time::SystemTime::now()
								.duration_since(std::time::UNIX_EPOCH)?
								.as_secs()
						);
						log::info!("Coping the old corrupted db to {}", db_bak);
						let _ = std::fs::copy("games.json", db_bak)
							.map_err(|e| log::error!("Error copying: {}", e));
					} else {
					*/
					return Err(e.into());
					//}
				}
			}
		} else {
			games = Vec::new();
		}

		// sort games by name
		games.sort_by(|(a, _), (b, _)| a.cmp(b));
		Ok(Self {
			games,
			current_game_num: None,
		})
	}

	pub fn add_game(&mut self, name: String) -> OrderNum {
		self.games.push((name, GameState::default()));
		OrderNum(self.games.len() - 1)
	}

	pub fn get_names(&self) -> Vec<&str> {
		self.games.iter().map(|(name, _)| name.as_str()).collect()
	}

	pub fn set_current_game_num(&mut self, num: OrderNum) {
		assert!(*num < self.games.len());

		self.current_game_num = Some(num);
		let mut state = &mut self.games[*num].1;
		if !state.players.is_empty() && state.order.is_empty() {
			state.order = state.players.iter().map(|(id, _)| *id).collect();
		}
	}

	pub fn get_current_game_state(&mut self) -> Option<&mut GameState> {
		self.games
			.get_mut(*self.current_game_num?)
			.map(|x| &mut x.1)
	}

	pub fn save(&self) -> Result<()> {
		log::debug!("Saving game data to the db");
		std::fs::write("games.json", serde_json::to_string(&self.games)?).map_err(|e| {
			log::error!("Error saving game data to the db: {}", e);
			e
		})?;

		Ok(())
	}

	/*
	pub fn add_debug_game(&mut self) {
		/*
		 * TODO: Move this logic into a separate menu option
		state.stat_list = {
			let mut map = HashMap::new();
			map.insert(Uid(0), "Strength".to_string());
			map.insert(Uid(1), "Dexterity".to_string());
			map.insert(Uid(2), "Poise".to_string());
			map.insert(Uid(3), "Wisdom".to_string());
			map.insert(Uid(4), "Intelligence".to_string());
			map.insert(Uid(5), "Charisma".to_string());
			StatList::new(map)
		};

		state.status_list = {
			let mut map = HashMap::new();
			map.insert(Uid(0), "Discharge".to_string());
			map.insert(Uid(1), "Fire Attack".to_string());
			map.insert(Uid(2), "Fire Shield".to_string());
			map.insert(Uid(3), "Ice Shield".to_string());
			map.insert(Uid(4), "Blizzard".to_string());
			map.insert(Uid(5), "Fusion".to_string());
			map.insert(Uid(6), "Luck".to_string());
			map.insert(Uid(7), "Knockdown".to_string());
			map.insert(Uid(8), "Poison".to_string());
			map.insert(Uid(9), "Stun".to_string());
			StatusList::new(map)
		};

		if debug_add {
			let mut players = Players::default();
			let mut stat_list = StatList::default();
			let mut status_list = StatusList::default();

			for i in 0..5 {
				let mut skills = Vec::new();
				let mut stats = Stats::default();
				let mut statuses = Statuses::default();
				for k in 0..4 {
					skills.push(Skill::new(format!("Testing skill {}", i), k * k));
					stats.set(format!("Testing stat {}", i), (i * k) as i32);
					statuses.push(Status::new(
						format!("Testing status {}", i),
						StatusCooldownType::Manual,
						k * k,
					));
				}
				let mut player = Player::new(format!("Testing player #{}", i), skills);
				player.stats = stats;
				player.statuses = statuses;
				players.push(player);
				stat_list.insert(format!("Testing stat {}", i));
				status_list.insert(format!("Testing status {}", i));
			}

			let game_state = GameState {
				order: players.iter().map(|(id, _)| *id).collect::<Vec<Uid>>(),
				players,
				stat_list,
				status_list,
			};

			games.insert(0, ("DEBUG".to_string(), game_state));
		}
		*/
	}
	*/
}
