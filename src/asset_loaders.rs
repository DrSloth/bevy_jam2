pub mod maps;

use std::io::Cursor;
use std::path::Path;
use bevy::asset::{Assets, Handle};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::ImageSampler;
use image::{ImageFormat, Rgba32FImage};
use rust_embed::RustEmbed;
use image::io::Reader as ImageReader;
use thiserror::Error;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct EmbeddedAssets;

#[derive(RustEmbed)]
#[folder = "data/"]
pub struct EmbeddedData;

pub trait EmbeddedAssetLoader {
    fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AssetLoadError>;
    fn load_image_as_asset<P: AsRef<Path>>(assets: &mut Assets<Image>, path: P) -> Result<Handle<Image>, AssetLoadError>;
    fn load_image<P: AsRef<Path>>(path: P) -> Result<Rgba32FImage, AssetLoadError>;
}

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("The given asset was not found: {0}")]
    NotFound(String),
    #[error("The given path was invalid unicode")]
    InvalidPath,
    #[error("The given image asset could not be decoded")]
    DecodeImageError,
}

impl<T: RustEmbed> EmbeddedAssetLoader for T {
    fn load<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, AssetLoadError> {
        let path = path.as_ref()
            .as_os_str()
            .to_str()
            .ok_or(AssetLoadError::InvalidPath)?;
        T::get(path)
            .ok_or_else(|| AssetLoadError::NotFound(path.to_string()))
            .map(|f| f.data.to_vec())
    }

    /// TODO: Optimize
    fn load_image_as_asset<P: AsRef<Path>>(assets: &mut Assets<Image>, path: P) -> Result<Handle<Image>, AssetLoadError> {
        Self::load_image(path).map(|image| {
            let mut texture = Image::new(
                Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                // darken_image(conv.into_raw()).into_iter().map(|f| f.to_ne_bytes()).flatten().collect(),
                image.into_raw().into_iter().map(|f| f.to_ne_bytes()).flatten().collect(),
                TextureFormat::Rgba32Float,
            );
            texture.sampler_descriptor = ImageSampler::nearest();
            assets.add(texture)
        })
    }

    fn load_image<P: AsRef<Path>>(path: P) -> Result<Rgba32FImage, AssetLoadError> {
        let mut image = ImageReader::new(
            Cursor::new(Self::load(path)?)
        );
        image.set_format(ImageFormat::Png);
        Ok(image.decode().map_err(|_| AssetLoadError::DecodeImageError)?.into_rgba32f())
    }
}

// fn darken_image(bytes: Vec<f32>) -> Vec<f32> {
//     bytes.chunks(4).map(|mut c| {
//         let mut pixel = [0.0; 4];
//         for (pixel, c) in pixel.iter_mut().take(3).zip(c) {
//             *pixel = (*c - 0.25).max(0.0);
//         }
//         pixel[3] = c[3];
//         pixel
//     }).flatten().collect()
// }
