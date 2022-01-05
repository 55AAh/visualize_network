use serde::Deserialize;
use serde_json;
use std::collections::VecDeque;
use std::io::{BufReader, Read};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Scenario {
    pub nodes: Vec<(i32, i32, bool)>,
    pub cable_connections: Vec<(usize, usize)>,
    pub transmissions: VecDeque<(u32, Uuid, usize, usize)>,
}
impl Scenario {
    pub fn load<T: Read>(reader: BufReader<T>) -> Result<Self, String> {
        let scenario: Scenario =
            serde_json::from_reader(reader).map_err(|err| format!("Serde error: {}", err))?;

        Ok(scenario)
    }
}
