use std::error::Error;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ComfortConfig {
    pub terrainperlin: FractalSettings,
    pub treeperlin: FractalSettings,
}

#[derive(Deserialize, Debug)]
pub struct FractalSettings {
    pub octaves: i32,
    pub gain: f32,
    pub lacunarity: f32,
    pub frequency: f32,
}

pub fn load_settings(preset: &str) -> Result<FractalSettings, Box<dyn Error>> {
    let contents = fs::read_to_string("config/config.toml")?;
    let decoded: ComfortConfig = toml::from_str(&contents).unwrap();
    match preset {
        "terrainperlin" => Ok(decoded.terrainperlin),
        "treeperlin" => Ok(decoded.treeperlin),
        _ => Ok(decoded.terrainperlin),
    }
}

