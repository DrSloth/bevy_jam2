pub mod maps;

use std::path::Path;
use rust_embed::RustEmbed;
use thiserror::Error;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct EmbeddedAssets;

#[derive(RustEmbed)]
#[folder = "data/"]
pub struct EmbeddedData;

pub trait EmbeddedAssetLoader {
    fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AssetLoadError>;
}

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("The given asset was not found")]
    NotFound,
}

impl<T: RustEmbed> EmbeddedAssetLoader for T {
    fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AssetLoadError> {
        path.as_ref()
            .as_os_str()
            .to_str()
            .and_then(T::get)
            .map(|f| f.data.to_vec())
            .ok_or(AssetLoadError::NotFound)
    }
}
