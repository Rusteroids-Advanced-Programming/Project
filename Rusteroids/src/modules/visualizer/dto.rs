use crate::modules::read_galaxy::stats::{ExplorerDataDTO, PlanetDataDTO};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct GalaxyResponse {
    pub(crate) planets: Vec<PlanetDataDTO>,
    pub(crate) explorers: Vec<ExplorerDataDTO>,
    pub game_won: bool,
    pub winner_id: Option<u32>,
}
