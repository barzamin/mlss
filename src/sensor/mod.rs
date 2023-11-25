use std::time::Duration;

use crate::I2cBus;

mod pmsa003i;
mod scd30;
mod sht40;
pub use self::pmsa003i::Pmsa003i;
pub use self::scd30::Scd30;
pub use self::sht40::Sht40;

pub trait Sensor: Sized {
    const NAME: &'static str;

    fn boot(bus: &'static I2cBus) -> anyhow::Result<Self>;
    fn poll(&mut self) -> anyhow::Result<()>;
    fn poll_period(&self) -> Duration;
}

#[derive(Copy, Clone)]
pub struct Manager {
    pub i2cbus: &'static I2cBus,
}

impl Manager {
    pub async fn run<S: Sensor>(self) -> anyhow::Result<()> {
        // TODO: retry boot etc.
        log::info!("booting {}...", S::NAME);
        let mut sensor = S::boot(self.i2cbus)?;

        let mut poll_wait = tokio::time::sleep(Duration::ZERO);
        loop {
            poll_wait.await;

            match sensor.poll() {
                Ok(()) => {
                    poll_wait = tokio::time::sleep(sensor.poll_period());
                }
                Err(error) => {
                    log::warn!("error polling {}: {:?}", S::NAME, error);
                    // TODO: backoff
                    poll_wait = tokio::time::sleep(sensor.poll_period());
                }
            }
        }

        Ok(())
    }
}
