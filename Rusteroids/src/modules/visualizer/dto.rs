use serde::Serialize;
use crate::modules::read_galaxy::stats::{ExplorerDataDTO, PlanetDataDTO};

#[derive(Serialize)]
pub(crate) struct GalaxyResponse {
    pub(crate) planets: Vec<PlanetDataDTO>,
    pub(crate) explorers: Vec<ExplorerDataDTO>,
}