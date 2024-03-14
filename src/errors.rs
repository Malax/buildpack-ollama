use libherokubuildpack::download::DownloadError;
use std::process::ExitStatus;

#[derive(thiserror::Error, Debug)]
pub(crate) enum OllamaBuildpackError {
    #[error("Could not start Ollama server: {0}")]
    OllamaServeIoError(std::io::Error),
    #[error("Ollama server does not respond: {0}")]
    OllamaServerUnresponsive(Box<ureq::Error>),
    #[error("Could not pull Ollama model: {0}")]
    OllamaPullIoError(std::io::Error),
    #[error("Could not pull Ollama model, unexpected exit status: {0}")]
    OllamaPullExitStatus(ExitStatus),
    #[error("Could not create Ollama model: {0}")]
    OllamaCreateIoError(std::io::Error),
    #[error("Could not create Ollama model, unexpected exit status: {0}")]
    OllamaCreateExitStatus(ExitStatus),
    #[error("Unexpected IO error: {0}")]
    GenericIoError(std::io::Error),
    #[error("Error while downloading Ollama: {0}")]
    OllamaDownloadError(DownloadError),
    #[error("Ollama Modelfile does not contain a base model")]
    MissingBaseModel,
    #[error("Target architecture not supported")]
    UnsupportedArchitecture,
}

impl From<OllamaBuildpackError> for libcnb::Error<OllamaBuildpackError> {
    fn from(value: OllamaBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}
