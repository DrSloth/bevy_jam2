use bevy::{asset::Handle, prelude::*, utils::HashMap};
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use crate::asset_loaders::{AssetLoadError, EmbeddedAssetLoader};

pub struct AssetCache<T: EmbeddedAssetLoader> {
    _phantom: PhantomData<fn(T) -> T>,
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
                // println!("Getting {:?} from cache", path.as_ref());
                Ok(assets.get_handle(handle))
            }
            None => {
                // println!("Loading {:?} to cache", path.as_ref());
                let handle = T::load_image_as_asset(assets, path.as_ref())?;
                let h = assets.get_handle(&handle);
                self.cache.insert(path.as_ref().to_path_buf(), handle);
                Ok(h)
            }
        }
    }
}
