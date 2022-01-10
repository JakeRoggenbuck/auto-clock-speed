use toml::de;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub powersave_under: i32,
    pub charging_powersave_under: i32,
}
