use gpui::{AssetSource, Result, SharedString};
use rust_embed::Embed;
use std::borrow::Cow;

#[derive(Embed)]
#[folder = "assets/icons/"]
#[prefix = "icons/"]
#[include = "*.svg"]
#[include = "**/*.svg"]
#[include = "blank.ico"]
#[include = "flags/{ru,us}.png"]
pub struct EmbeddedAssetSource;

impl AssetSource for EmbeddedAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let clean_path = path.trim_start_matches('/');
        Ok(Self::get(clean_path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let clean_path = path.trim_matches('/');
        let prefix = if clean_path.is_empty() {
            String::new()
        } else {
            format!("{clean_path}/")
        };

        Ok(Self::iter()
            .filter_map(|asset| {
                let relative = asset.strip_prefix(&prefix)?;
                (!relative.contains('/')).then(|| relative.to_string().into())
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assets_do_not_depend_on_working_directory() {
        let original_dir = std::env::current_dir().expect("current directory should be available");
        std::env::set_current_dir(std::env::temp_dir())
            .expect("temporary directory should be available");

        let icons = [
            "icons/house.svg",
            "icons/chart-bar-stacked.svg",
            "icons/arrow-left-right.svg",
            "icons/gamepad-2.svg",
            "icons/circle-slash.svg",
            "icons/headphones.svg",
            "icons/tv.svg",
            "icons/trophy.svg",
            "icons/sliders-horizontal.svg",
        ];
        let embedded = icons.map(|path| {
            EmbeddedAssetSource
                .load(path)
                .expect("asset lookup should succeed")
        });

        std::env::set_current_dir(original_dir).expect("current directory should be restored");
        assert!(
            embedded.into_iter().all(|icon| icon.is_some()),
            "release assets must be embedded"
        );
    }
}
