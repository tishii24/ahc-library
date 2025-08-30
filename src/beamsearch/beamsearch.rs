pub struct BeamSearchConfig {
    pub beam_width: usize,
}

pub struct BeamSearch {
    pub config: BeamSearchConfig,
}

impl BeamSearch {
    pub fn new(config: BeamSearchConfig) -> BeamSearch {
        BeamSearch { config }
    }

    pub fn run(&self) {
        todo!()
    }
}
