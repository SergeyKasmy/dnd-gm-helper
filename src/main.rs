#![feature(try_blocks)]

use crate::client::Client;

use anyhow::Result;
use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::fs::OpenOptions;

mod client;
mod ui;

fn main() -> Result<()> {
	WriteLogger::init(
		LevelFilter::Trace,
		Config::default(),
		OpenOptions::new()
			.create(true)
			.append(true)
			.open("dnd.log")?,
	)?;
	Client::new()?.run()
}
