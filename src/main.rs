use log::LevelFilter;
use simplelog::{WriteLogger, Config};
use std::fs::OpenOptions;

fn main() {
    WriteLogger::init(LevelFilter::Debug, Config::default(), OpenOptions::new().create(true).append(true).open("dnd.log").unwrap()).unwrap();
    dnd_gm_helper::run();
}
