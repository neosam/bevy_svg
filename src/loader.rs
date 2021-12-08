use anyhow;
use bevy::{asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset}, prelude::info};
use thiserror::Error;

use crate::prelude::Svg;


#[derive(Default)]
pub struct SvgAssetLoader;

impl AssetLoader for SvgAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut opts = usvg::Options::default();
            opts.fontdb.load_system_fonts();
            opts.fontdb.load_fonts_dir("./assets");

            info!("Parsing SVG: {}", load_context.path().display());
            let svg_tree = usvg::Tree::from_data(&bytes, &opts.to_ref()).map_err(|err| {
                FileSvgError {
                    error: err.into(),
                    path: format!("{}", load_context.path().display()),
                }
            })?;

            let mut svg = Svg::from_tree(svg_tree);
            let name = &load_context.path().file_name().ok_or_else(||
                FileSvgError {
                    error: SvgError::InvalidFileName(load_context.path().display().to_string()),
                    path: format!("{}", load_context.path().display()),
                }
            )?.to_string_lossy();
            svg.name = name.to_string();

            load_context.set_default_asset(LoadedAsset::new(svg));
            info!("Parsing SVG: {} ... Done", load_context.path().display());
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["svg", "svgz"]
    }
}

/// An error that occurs when loading a texture
#[derive(Error, Debug)]
pub enum SvgError {
    #[error("invalid file name")]
    InvalidFileName(String),
    #[error("failed to load an SVG: {0}")]
    SvgError(#[from] usvg::Error),
}

/// An error that occurs when loading a texture from a file.
#[derive(Error, Debug)]
pub struct FileSvgError {
    error: SvgError,
    path: String,
}
impl std::fmt::Display for FileSvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "Error reading SVG file {}: {}, this is an error in `bevy_svg`.",
            self.path, self.error
        )
    }
}
