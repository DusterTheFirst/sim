use timescale::{Lerp, TimescaleData, TimescaleDataTable};

#[derive(Lerp, TimescaleData)]
#[timescale(rename = "Time (N)")]
pub struct RocketEngine {
    #[timescale(rename = "Thrust (N)")]
    thrust: f64,
}

#[derive(TimescaleDataTable)]
#[table(file = "csv/Estes_C6.csv", st = RocketEngine)]
pub struct EstesC6;
