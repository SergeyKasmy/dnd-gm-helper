use anyhow::Result;
use dnd_gm_helper::server::Server;

pub fn run() -> Result<()> {
	/*
	use std::panic;

	log::debug!("Starting...");
	log_panics::init();
	// TODO: do something about it
	if let Err(e) = panic::catch_unwind(start) {
		if let Ok(term) = Term::new() {
			let _ = term.messagebox("sowwy! OwO the pwogwam cwashed! 🥺 pwease d-don't bwame the d-devewopew, òωó he's d-doing his best!");
		}
		panic::resume_unwind(e);
	}
	Ok(())
	*/

	let term = Term::new()?;

    let mut server = Server::new()?;

	let game_num = {
		let mut options = games
			.iter()
			.map(|(name, _)| name.as_str())
			.collect::<Vec<&str>>();
		options.push("Add...");
		loop {
			match term.messagebox_with_options("Choose the game", &options, true)? {
				Some(num) => {
					if num >= games.len().into() {
						let name =
							term.messagebox_with_input_field("Enter the name of the new game")?;
						games.push((name, GameState::default()));
					}
					break num;
				}
				None => return Ok(()),
			}
		}
	};
    server.set_current_game_num(game_num);

	Ok(())
}
