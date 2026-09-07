use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub const HOURS_PER_DAY: u64 = 24;
pub const TICKS_PER_HOUR: u64 = 1;

#[derive(Clone, Debug)]
pub struct SimulationClock {
    pub tick: u64,
    pub speed: f64,
    pub rng: StdRng,
}

impl SimulationClock {
    pub fn new(seed: u64, speed: f64) -> Self {
        Self {
            tick: 0,
            speed,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    pub fn advance(&mut self, ticks: u64) {
        self.tick += ticks;
    }

    pub fn random(&mut self) -> f64 {
        self.rng.gen()
    }

    pub fn hour_of_day(&self) -> u64 {
        self.tick % HOURS_PER_DAY
    }

    pub fn day(&self) -> u64 {
        (self.tick / HOURS_PER_DAY) + 1
    }

    pub fn is_night(&self) -> bool {
        let h = self.hour_of_day();
        h >= 22 || h < 6
    }

    pub fn is_work_hours(&self) -> bool {
        let h = self.hour_of_day();
        h >= 9 && h < 18
    }
}
