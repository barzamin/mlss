use std::time::Duration;

use metrics::{register_gauge, Gauge};
use pmsa003i::{Concentrations, ParticleCounts};

use crate::I2cRef;

use super::Sensor;

const NAME: &'static str = "PMSA003I";

struct ConcentrationGauges {
    pm1_0: Gauge,
    pm1_0_standard: Gauge,
    pm2_5: Gauge,
    pm2_5_standard: Gauge,
    pm10_0: Gauge,
    pm10_0_standard: Gauge,
}

impl ConcentrationGauges {
    pub fn new() -> Self {
        const GAUGE: &'static str = "pm_conc";
        Self {
            pm1_0: register_gauge!(GAUGE, "sensor" => NAME, "pm" => "1.0", "cond" => "env"),
            pm1_0_standard: register_gauge!(GAUGE, "sensor" => NAME, "pm" => "1.0", "cond" => "std"),
            pm2_5: register_gauge!(GAUGE, "sensor" => NAME, "pm" => "2.5", "cond" => "env"),
            pm2_5_standard: register_gauge!(GAUGE, "sensor" => NAME, "pm" => "2.5", "cond" => "std"),
            pm10_0: register_gauge!(GAUGE, "sensor" => NAME, "pm" => "10.0", "cond" => "env"),
            pm10_0_standard: register_gauge!(GAUGE, "sensor" => NAME, "pm" => "10.0", "cond" => "std"),
        }
    }

    pub fn update(&self, meas: Concentrations) {
        self.pm1_0.set(meas.pm1_0 as f64);
        self.pm1_0_standard.set(meas.pm1_0_standard as f64);
        self.pm2_5.set(meas.pm2_5 as f64);
        self.pm2_5_standard.set(meas.pm2_5_standard as f64);
        self.pm10_0.set(meas.pm10_0 as f64);
        self.pm10_0_standard.set(meas.pm10_0_standard as f64);
    }
}

struct ParticleCountGauges {
    /// Number of particles with diameter >= 0.3 µm in 0.1L of air.
    pub particles_0_3um: Gauge,
    /// Number of particles with diameter >= 0.5 µm in 0.1L of air.
    pub particles_0_5um: Gauge,
    /// Number of particles with diameter >= 1.0 µm in 0.1L of air.
    pub particles_1_0um: Gauge,
    /// Number of particles with diameter >= 2.5 µ𝑚 in 0.1L of air.
    pub particles_2_5um: Gauge,
    /// Number of particles with diameter >= 5.0 µm in 0.1L of air.
    pub particles_5_0um: Gauge,
    /// Number of particles with diameter >= 10.0 µm in 0.1L of air.
    pub particles_10_0um: Gauge,
}

impl ParticleCountGauges {
    pub fn new() -> Self {
        const GAUGE: &'static str = "particle_count";
        Self {
            particles_0_3um: register_gauge!(GAUGE, "sensor" => NAME, "diam" => "0.3"),
            particles_0_5um: register_gauge!(GAUGE, "sensor" => NAME, "diam" => "0.5"),
            particles_1_0um: register_gauge!(GAUGE, "sensor" => NAME, "diam" => "1.0"),
            particles_2_5um: register_gauge!(GAUGE, "sensor" => NAME, "diam" => "2.5"),
            particles_5_0um: register_gauge!(GAUGE, "sensor" => NAME, "diam" => "5.0"),
            particles_10_0um: register_gauge!(GAUGE, "sensor" => NAME, "diam" => "10.0"),
        }
    }

    pub fn update(&self, meas: ParticleCounts) {
        self.particles_0_3um.set(meas.particles_0_3um as f64);
        self.particles_0_5um.set(meas.particles_0_5um as f64);
        self.particles_1_0um.set(meas.particles_1_0um as f64);
        self.particles_2_5um.set(meas.particles_2_5um as f64);
        self.particles_5_0um.set(meas.particles_5_0um as f64);
        self.particles_10_0um.set(meas.particles_10_0um as f64);
    }
}

pub struct Pmsa003i {
    sensor: pmsa003i::Pmsa003i<I2cRef<'static>>,
    pm_gauges: ConcentrationGauges,
    count_gauges: ParticleCountGauges,
}

impl Sensor for Pmsa003i {
    const NAME: &'static str = NAME;

    fn boot(bus: &'static crate::I2cBus) -> anyhow::Result<Self> {
        log::info!("connecting to {}", Self::NAME);
        let i2c = bus.acquire_i2c();
        let sensor = pmsa003i::Pmsa003i::new(i2c);

        Ok(Self {
            sensor,
            pm_gauges: ConcentrationGauges::new(),
            count_gauges: ParticleCountGauges::new(),
        })
    }

    fn poll(&mut self) -> anyhow::Result<()> {
        let meas = self
            .sensor
            .read()
            .map_err(|error| anyhow::anyhow!("error reading from {}: {:?}", Self::NAME, error))?;

        log::debug!("successful read from {}: {}", Self::NAME, meas);

        self.pm_gauges.update(meas.concentrations);
        self.count_gauges.update(meas.counts);

        Ok(())
    }

    fn poll_period(&self) -> std::time::Duration {
        Duration::from_secs(2)
    }
}
