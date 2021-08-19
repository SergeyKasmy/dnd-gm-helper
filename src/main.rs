#![feature(try_blocks)]

use anyhow::Result;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs::OpenOptions;

mod client;
mod term;

fn main() -> Result<()> {
	WriteLogger::init(
		LevelFilter::Trace,
		Config::default(),
		OpenOptions::new()
			.create(true)
			.append(true)
			.open("dnd.log")?,
	)?;
	client::run()
}
