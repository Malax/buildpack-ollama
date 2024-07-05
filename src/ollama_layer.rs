use crate::errors::OllamaBuildpackError;
use crate::util::set_executable;
use crate::OllamaBuildpack;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::{additional_buildpack_binary_path, Buildpack};
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
            cache: false,
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
        .exec_d_program(
            "ollama_env",
            additional_buildpack_binary_path!("ollama_env"),
        )
        .build()
    }
}
