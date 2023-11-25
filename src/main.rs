use std::{net::SocketAddr, sync::Mutex, thread, time::Duration};

mod noaa;
mod sensor;

use anyhow::Error;
use futures::future::join_all;
use linux_embedded_hal::{Delay, I2cdev};
use log::debug;
use metrics::gauge;
use metrics_exporter_prometheus::PrometheusBuilder;
use pmsa003i::Pmsa003i;
use sensor_scd30::Scd30;
use shared_bus::BusManager;
use tokio::runtime::Runtime;

pub type SharedI2c = Mutex<I2cdev>;
pub type I2cBus = BusManager<SharedI2c>;
pub type I2cRef<'bus> = shared_bus::I2cProxy<'bus, SharedI2c>;

fn main() -> Result<(), Error> {
    env_logger::init();

    let rt = Runtime::new()?;

    let i2c = I2cdev::new("/dev/i2c-1")?;
    let i2cbus: &'static I2cBus =
        shared_bus::new_std!(I2cdev = i2c).expect("cant construct shared i2c bus");

    let noaa_obsv = rt.block_on(noaa::fetch_observations("KNYC"))?;

    PrometheusBuilder::new()
        .with_http_listener("0.0.0.0:9000".parse::<SocketAddr>().unwrap())
        .install()?;

    let mgr = sensor::Manager { i2cbus };

    let mut handles = vec![];
    handles.push(rt.spawn(mgr.run::<sensor::Scd30>()));
    handles.push(rt.spawn(mgr.run::<sensor::Pmsa003i>()));
    handles.push(rt.spawn(mgr.run::<sensor::Sht40>()));

    rt.block_on(async {
        join_all(handles).await;
    });

    Ok(())
}
