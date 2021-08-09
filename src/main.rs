use anyhow::Result;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs::OpenOptions;

fn main() -> Result<()> {
	WriteLogger::init(
		LevelFilter::Debug,
		Config::default(),
		OpenOptions::new()
			.create(true)
			.append(true)
			.open("dnd.log")?,
	)?;
	dnd_gm_helper::run()
}
