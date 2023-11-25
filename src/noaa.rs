use anyhow::Error;
use log::debug;
use reqwest::Client;

#[derive(Debug)]
pub struct Observation {
    /// °C
    pub temperature: f64,
    /// %
    pub relative_humidity: f64,
    /// Pa
    pub barometric_pressure: f64,
}

pub async fn fetch_observations(station: &str) -> Result<Observation, Error> {
    let res = Client::builder()
        .user_agent("MLSS/0.1 (erin@hecke.rs)")
        .build()?
        .get(format!(
            "https://api.weather.gov/stations/{}/observations/latest",
            station
        ))
        .send()
        .await?
        .error_for_status()?;

    let data = res.json::<serde_json::Value>().await?;

    Ok(Observation {
        temperature: data["properties"]["temperature"]["value"]
            .as_f64()
            .expect("f64"),
        relative_humidity: data["properties"]["relativeHumidity"]["value"]
            .as_f64()
            .expect("f64"),
        barometric_pressure: data["properties"]["barometricPressure"]["value"]
            .as_f64()
            .expect("f64"),
    })
}
