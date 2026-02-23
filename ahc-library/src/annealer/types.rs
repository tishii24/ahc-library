use crate::utils::random::Random;

pub trait Criterion {
    fn adopt(
        &self,
        cur_score: f64,
        new_score: f64,
        cur_temp: f64,
        progress: f64,
        rnd: &mut impl Random,
    ) -> bool;
}

pub trait TemperatureScheduler {
    fn get_temp(&self, progress: f64) -> f64;
}

pub trait ProgressScheduler {
    fn start(&mut self) {}
    fn step(&mut self) {}
    fn get_progress(&self) -> f64;
}
