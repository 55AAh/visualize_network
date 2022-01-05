#![windows_subsystem = "windows"]

mod app;
mod network;

use app::{scenario::Scenario, App};
use clap;
use std::fs::File;
use std::io::{stdin, BufReader};

pub fn main() -> Result<(), String> {
    let matches = clap::App::new("Computer Network Visualizer")
        .version("0.1.0")
        .author("Kostiantyn Kulyk <kostia.kulik@gmail.com>")
        .arg(
            clap::Arg::new("file")
                .short('f')
                .long("file")
                .takes_value(true)
                .allow_invalid_utf8(true)
                .help("Read JSON-formatted simulation scenario from a file"),
        )
        .arg(
            clap::Arg::new("stdin")
                .short('s')
                .long("stdin")
                .conflicts_with("file")
                .help("Read scenario from stdin"),
        )
        .get_matches();

    let scenario = if matches.is_present("stdin") {
        let reader = BufReader::new(stdin());
        Some(Scenario::load(reader).expect("Unable to load scenario from file!"))
    } else {
        matches.value_of_os("file").and_then(|path| {
            let file = File::open(path).expect("Cannot read file!");
            let reader = BufReader::new(file);
            Some(Scenario::load(reader).expect("Unable to load scenario from file!"))
        })
    };
    App::new()?.run(scenario)
}
