pub mod maps;

use bevy::{
    asset::{Assets, Handle},
    prelude::*,
    render::texture::ImageSampler,
    render::{
        // render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::{CompressedImageFormats, ImageType},
    },
};
use std::{io::Cursor, path::Path};

use image::{io::Reader as ImageReader, DynamicImage, ImageBuffer, ImageFormat, Pixel, Rgba};
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
    fn load_image_as_asset<P: AsRef<Path>>(
        assets: &mut Assets<Image>,
        path: P,
    ) -> Result<Handle<Image>, AssetLoadError>;
    fn load_image<P: AsRef<Path>, I: Pixel + ImageConverter>(
        path: P,
    ) -> Result<I::Buffer, AssetLoadError>;
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
        let path = path
            .as_ref()
            .as_os_str()
            .to_str()
            .ok_or(AssetLoadError::InvalidPath)?;
        T::get(path)
            .ok_or_else(|| AssetLoadError::NotFound(path.to_owned()))
            .map(|f| f.data.to_vec())
    }

    /// TODO: Optimize
    fn load_image_as_asset<P: AsRef<Path>>(
        assets: &mut Assets<Image>,
        path: P,
    ) -> Result<Handle<Image>, AssetLoadError> {
        let path = path.as_ref();
        let image = Self::load(path)?;
        let mut image = Image::from_buffer(
            &image,
            ImageType::MimeType("image/png"),
            CompressedImageFormats::empty(),
            true,
        )
        .unwrap_or_else(|e| panic!("Image {:?} couldn't be loaded: {}", path.to_str(), e));
        image.sampler_descriptor = ImageSampler::nearest();
        Ok(assets.add(image))
    }

    fn load_image<P: AsRef<Path>, I: Pixel + ImageConverter>(
        path: P,
    ) -> Result<I::Buffer, AssetLoadError> {
        let mut image = ImageReader::new(Cursor::new(Self::load(path)?));
        image.set_format(ImageFormat::Png);
        Ok(I::conv(
            image
                .decode()
                .map_err(|_| AssetLoadError::DecodeImageError)?,
        ))
    }
}

pub trait ImageConverter {
    type Buffer;
    fn conv(image: DynamicImage) -> Self::Buffer;
}

impl ImageConverter for Rgba<f32> {
    type Buffer = ImageBuffer<Rgba<f32>, Vec<f32>>;
    fn conv(image: DynamicImage) -> Self::Buffer {
        image.to_rgba32f()
    }
}

impl ImageConverter for Rgba<u8> {
    type Buffer = ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn conv(image: DynamicImage) -> Self::Buffer {
        image.to_rgba8()
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
