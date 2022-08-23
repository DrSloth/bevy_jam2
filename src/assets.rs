mod embed_io;
mod maps;

use std::ops::Deref;
use rust_embed::RustEmbed;
use bevy::prelude::*;
use bevy::asset::{AssetServer, FileAssetIo};
use crate::assets::embed_io::EmbedIo;

#[derive(RustEmbed)]
#[folder = "assets/"]
pub struct EmbeddedAssets;

#[derive(RustEmbed)]
#[folder = "data/"]
pub struct EmbeddedData;

pub struct AssetsAssetServer(AssetServer);

impl Deref for AssetsAssetServer {
    type Target = AssetServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DataAssetServer(AssetServer);

impl Deref for DataAssetServer {
    type Target = AssetServer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_asset_servers<'a, 'b>(mut commands: Commands<'a, 'b>) -> Commands<'a, 'b> {
    commands.insert_resource(DataAssetServer(
        if cfg!(debug_assertions) {
            AssetServer::new(
                FileAssetIo::new("data", true)
            )
        } else {
            AssetServer::new(EmbedIo(EmbeddedData))
        }
    ));
    commands.insert_resource(AssetsAssetServer(
        if cfg!(debug_assertions) {
            AssetServer::new(
                FileAssetIo::new("assets", true)
            )
        } else {
            AssetServer::new(EmbedIo(EmbeddedAssets))
        }
    ));
    commands
}
