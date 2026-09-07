use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub const MINUTES_PER_TICK: u64 = 1;
pub const TICKS_PER_HOUR: u64 = 60;
pub const HOURS_PER_DAY: u64 = 24;
pub const TICKS_PER_DAY: u64 = TICKS_PER_HOUR * HOURS_PER_DAY;
pub const DAYS_PER_WEEK: u64 = 7;
pub const DAYS_PER_MONTH: u64 = 30;
pub const MONTHS_PER_YEAR: u64 = 12;
pub const TICKS_PER_WEEK: u64 = TICKS_PER_DAY * DAYS_PER_WEEK;
pub const TICKS_PER_MONTH: u64 = TICKS_PER_DAY * DAYS_PER_MONTH;
pub const TICKS_PER_YEAR: u64 = TICKS_PER_MONTH * MONTHS_PER_YEAR;

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

    pub fn minute_of_hour(&self) -> u64 {
        self.tick % TICKS_PER_HOUR
    }

    pub fn hour_of_day(&self) -> u64 {
        (self.tick / TICKS_PER_HOUR) % HOURS_PER_DAY
    }

    pub fn day_of_week(&self) -> u64 {
        (self.tick / TICKS_PER_DAY) % DAYS_PER_WEEK
    }

    pub fn day_of_month(&self) -> u64 {
        (self.tick / TICKS_PER_DAY) % DAYS_PER_MONTH + 1
    }

    pub fn month(&self) -> u64 {
        (self.tick / TICKS_PER_MONTH) % MONTHS_PER_YEAR + 1
    }

    pub fn year(&self) -> u64 {
        self.tick / TICKS_PER_YEAR + 1
    }

    pub fn day(&self) -> u64 {
        (self.tick / TICKS_PER_DAY) + 1
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
