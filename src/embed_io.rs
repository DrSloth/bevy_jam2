//! File io for the Bevy Assets Server using rustembed

use std::path::{Path, PathBuf};

use bevy::asset::{AssetIo, AssetIoError, BoxedFuture, FileType, Metadata};
use rust_embed::RustEmbed;

/// Embedded io
#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct EmbedIo;

impl AssetIo for EmbedIo {
    fn load_path<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, anyhow::Result<Vec<u8>, AssetIoError>> {
        Box::pin(async move {
            path.as_os_str()
                .to_str()
                .and_then(Self::get)
                .map(|f| f.data.to_vec())
                .ok_or_else(|| AssetIoError::NotFound(PathBuf::from(path)))
        })
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> anyhow::Result<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        path.to_path_buf()
            .into_os_string()
            .into_string()
            .map(|path| {
                Self::iter()
                    .filter(move |s| s.starts_with(&path))
                    .map(|s| PathBuf::from(&*s))
            })
            .map(|iter| Box::new(iter) as Box<dyn Iterator<Item = PathBuf>>)
            .map_err(|s| AssetIoError::NotFound(PathBuf::from(s)))
    }

    fn get_metadata(&self, path: &Path) -> anyhow::Result<Metadata, AssetIoError> {
        path.as_os_str()
            .to_str()
            .map(Self::get)
            .map(|file| match file {
                Some(_) => FileType::File,
                None => FileType::Directory,
            })
            .map(Metadata::new)
            .ok_or_else(|| AssetIoError::NotFound(PathBuf::from(path)))
    }

    fn watch_path_for_changes(&self, _: &Path) -> anyhow::Result<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> anyhow::Result<(), AssetIoError> {
        Ok(())
    }
}
