use std::time::Duration;

use linux_embedded_hal::{i2cdev::linux::LinuxI2CError, Delay};
use metrics::{register_gauge, Gauge};

use crate::{I2cBus, I2cRef};

use super::Sensor;

pub struct Scd30 {
    sensor: sensor_scd30::Scd30<I2cRef<'static>, Delay, LinuxI2CError>,
    gauge_co2: Gauge,
    gauge_temp: Gauge,
    gauge_rh: Gauge,
}

impl Sensor for Scd30 {
    const NAME: &'static str = "SCD30";

    fn boot(bus: &'static I2cBus) -> anyhow::Result<Self> {
        log::info!("connecting to {}", Self::NAME);
        let i2c = bus.acquire_i2c();
        let mut sensor = sensor_scd30::Scd30::new(i2c, Delay {})
            .map_err(|error| anyhow::anyhow!("error initializing {}: {:?}", Self::NAME, error))?;
        sensor.start_continuous(0).map_err(|error| {
            anyhow::anyhow!("error starting {} measurements: {:?}", Self::NAME, error)
        })?; // TODO: pressure compensation (from NOAA?)

        Ok(Self {
            sensor,
            gauge_co2: register_gauge!("co2_ppm", "sensor" => Self::NAME),
            gauge_temp: register_gauge!("temp_degc", "sensor" => Self::NAME),
            gauge_rh: register_gauge!("rh_percent", "sensor" => Self::NAME),
        })
    }

    fn poll(&mut self) -> anyhow::Result<()> {
        if self.sensor.data_ready().map_err(|error| {
            anyhow::anyhow!("error checking {} readiness: {:?}", Self::NAME, error)
        })? {
            let meas = self.sensor.read_data().map_err(|error| {
                anyhow::anyhow!("error reading from {}: {:?}", Self::NAME, error)
            })?;

            log::debug!("successful read from {}: {:?}", Self::NAME, meas);

            self.gauge_co2.set(meas.co2 as f64);
            self.gauge_temp.set(meas.temp as f64);
            self.gauge_rh.set(meas.rh as f64);
        }

        Ok(())
    }

    fn poll_period(&self) -> Duration {
        Duration::from_secs(5)
    }
}
