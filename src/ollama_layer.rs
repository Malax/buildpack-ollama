use crate::errors::OllamaBuildpackError;
use crate::util::set_executable;
use crate::OllamaBuildpack;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::Buildpack;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub(crate) struct OllamaLayer {
    pub(crate) version: String,
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub(crate) struct OllamaLayerMetadata {
    version: String,
    url: String,
}

impl Layer for OllamaLayer {
    type Buildpack = OllamaBuildpack;
    type Metadata = OllamaLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            launch: true,
            build: true,
            cache: true,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        let bin_dir = layer_path.join("bin");
        let ollama_path = bin_dir.join("ollama");

        fs::create_dir_all(bin_dir).map_err(OllamaBuildpackError::GenericIoError)?;

        let download_url = self.url.clone();
        libherokubuildpack::log::log_header(format!("Installing Ollama from {}", download_url));
        libherokubuildpack::download::download_file(download_url.clone(), &ollama_path)
            .map_err(OllamaBuildpackError::OllamaDownloadError)?;

        set_executable(&ollama_path).map_err(OllamaBuildpackError::GenericIoError)?;

        LayerResultBuilder::new(OllamaLayerMetadata {
            version: self.version.clone(),
            url: download_url,
        })
        .env(LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Override,
            "OLLAMA_HOST",
            "0.0.0.0",
        ))
        .build()
    }

    fn existing_layer_strategy(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let expected_metadata = OllamaLayerMetadata {
            version: self.version.clone(),
            url: self.url.clone(),
        };

        Ok(
            if layer_data.content_metadata.metadata == expected_metadata {
                ExistingLayerStrategy::Keep
            } else {
                ExistingLayerStrategy::Recreate
            },
        )
    }
}
