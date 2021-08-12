use anyhow::Result;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs::OpenOptions;

mod client;
mod term;

fn main() -> Result<()> {
	WriteLogger::init(
		LevelFilter::Debug,
		Config::default(),
		OpenOptions::new()
			.create(true)
			.append(true)
			.open("dnd.log")?,
	)?;
	client::run()
}
