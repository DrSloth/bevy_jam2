use bevy::{asset::Handle, prelude::*, utils::HashMap};
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::asset_loaders::{AssetLoadError, EmbeddedAssetLoader};

pub struct AssetCache<T: EmbeddedAssetLoader> {
    _phantom: PhantomData<T>,
    cache: HashMap<PathBuf, Handle<Image>>,
}

impl<T: EmbeddedAssetLoader> AssetCache<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            cache: HashMap::new(),
        }
    }

    pub fn load_image<P: AsRef<Path>>(
        &mut self,
        assets: &mut Assets<Image>,
        path: P,
    ) -> Result<Handle<Image>, AssetLoadError> {
        match self.cache.get(path.as_ref()) {
            Some(handle) => {
                Ok(assets.get_handle(handle))
            }
            None => {
                let handle = T::load_image_as_asset(assets, path.as_ref())?;
                let h = assets.get_handle(&handle);
                self.cache.insert(path.as_ref().to_path_buf(), handle);
                Ok(h)
            }
        }
    }
    
    pub fn load_music<P: AsRef<Path>>(&mut self, audio_assets: &mut Assets<AudioSource>, path: P) -> Handle<AudioSource> {
        let audio = T::load(path).unwrap();
        audio_assets.add(AudioSource {
            bytes: audio.into()
        })
    }
}
