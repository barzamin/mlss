use std::time::Duration;

use linux_embedded_hal::Delay;
use metrics::{register_gauge, Gauge};
use sht4x::{Precision, Sht4x};

use crate::I2cRef;

use super::Sensor;

pub struct Sht40 {
    sensor: Sht4x<I2cRef<'static>, Delay>,
    gauge_temp: Gauge,
    gauge_rh: Gauge,
}

impl Sensor for Sht40 {
    const NAME: &'static str = "SHT40";

    fn boot(bus: &'static crate::I2cBus) -> anyhow::Result<Self> {
        let i2c = bus.acquire_i2c();
        let sensor = Sht4x::new(i2c);

        Ok(Self {
            sensor,
            gauge_temp: register_gauge!("temp_degc", "sensor" => Self::NAME),
            gauge_rh: register_gauge!("rh_percent", "sensor" => Self::NAME),
        })
    }

    fn poll(&mut self) -> anyhow::Result<()> {
        let meas = self
            .sensor
            .measure(Precision::High, &mut Delay {})
            .map_err(|error| anyhow::anyhow!("error reading from {}: {:?}", Self::NAME, error))?;

        log::debug!("successful read from {}: {:?}", Self::NAME, meas);

        self.gauge_temp.set(meas.temperature_celsius().unwrapped_to_num::<f64>());
        self.gauge_rh.set(meas.humidity_percent().unwrapped_to_num::<f64>());

        Ok(())
    }

    fn poll_period(&self) -> Duration {
        Duration::from_secs(10)
    }
}
