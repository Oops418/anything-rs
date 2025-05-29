use anyhow::anyhow;

use gpui::AssetSource;
use material_icon_embed_rs::Asset as MaterialAsset;
use rust_embed::RustEmbed;
use tracing::debug;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*"]
#[exclude = "*.DS_Store"]
pub struct LocalAsset;

pub struct VanillaAsset;

impl AssetSource for VanillaAsset {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        MaterialAsset::get(path)
            .map(|f| Some(f.data))
            .or_else(|| LocalAsset::get(path).map(|f| Some(f.data)))
            .ok_or_else(|| {
                debug!(
                    "Asset not found in both MaterialAsset and LocalAsset: {}",
                    path
                );
                anyhow!("Asset not found: {}", path)
            })
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let mut assets: Vec<gpui::SharedString> = MaterialAsset::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect();

        let local_assets: Vec<gpui::SharedString> = LocalAsset::iter()
            .filter_map(|p| {
                if p.starts_with(path) {
                    Some(p.into())
                } else {
                    None
                }
            })
            .collect();

        assets.extend(local_assets);
        assets.sort();
        assets.dedup();

        Ok(assets)
    }
}
