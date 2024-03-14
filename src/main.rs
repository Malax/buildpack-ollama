mod errors;
mod modelfile;
mod models_layer;
mod ollama_layer;
mod util;

use crate::errors::OllamaBuildpackError;
use crate::modelfile::base_model_from_modelfile;
use crate::models_layer::ModelsLayer;
use crate::ollama_layer::OllamaLayer;
use crate::util::wait_for_http_200;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericPlatform;
use libcnb::layer_env::Scope;
use libcnb::{buildpack_main, Buildpack, Env};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
struct OllamaBuildpackMetadata {
    ollama_version: String,
    amd64_download_url: String,
    arm64_download_url: String,
}

struct OllamaBuildpack;

impl Buildpack for OllamaBuildpack {
    type Platform = GenericPlatform;
    type Metadata = OllamaBuildpackMetadata;
    type Error = OllamaBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if context.app_dir.join("Modelfile").exists() {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let ollama_modelfile_path = context.app_dir.join("Modelfile");
        let ollama_base_model_name = base_model_from_modelfile(&ollama_modelfile_path)
            .map_err(OllamaBuildpackError::GenericIoError)?
            .ok_or(OllamaBuildpackError::MissingBaseModel)?;

        let mut env = Env::from_current();

        libherokubuildpack::log::log_header("Installing Ollama");
        let ollama_layer = context.handle_layer(
            layer_name!("ollama"),
            OllamaLayer {
                version: context.buildpack_descriptor.metadata.ollama_version.clone(),
                url: match context.target.arch.as_str() {
                    "arm64" => Ok(context
                        .buildpack_descriptor
                        .metadata
                        .arm64_download_url
                        .clone()),
                    "amd64" => Ok(context
                        .buildpack_descriptor
                        .metadata
                        .amd64_download_url
                        .clone()),
                    _ => Err(OllamaBuildpackError::UnsupportedArchitecture),
                }?,
            },
        )?;

        libherokubuildpack::log::log_info(format!(
            "Successfully installed Ollama {}!",
            &context.buildpack_descriptor.metadata.ollama_version
        ));

        env = ollama_layer.env.apply(Scope::Build, &env);

        let models_layer = context.handle_layer(layer_name!("models"), ModelsLayer)?;
        env = models_layer.env.apply(Scope::Build, &env);

        libherokubuildpack::log::log_header("Starting Ollama");
        let mut ollama_child_process = Command::new("ollama")
            .args(["serve"])
            .envs(&env)
            .spawn()
            .map_err(OllamaBuildpackError::OllamaServeIoError)?;

        let waited_duration = wait_for_http_200("http://localhost:11434/")
            .map_err(OllamaBuildpackError::OllamaServerUnresponsive)?;

        libherokubuildpack::log::log_info(format!(
            "Successfully started after {}ms",
            waited_duration.as_millis()
        ));

        libherokubuildpack::log::log_header(format!(
            "Pulling '{}' base model",
            ollama_base_model_name
        ));

        Command::new("ollama")
            .args(["pull", &ollama_base_model_name])
            .envs(&env)
            .spawn()
            .map_err(OllamaBuildpackError::OllamaPullIoError)?
            .wait()
            .map_err(OllamaBuildpackError::OllamaPullIoError)
            .and_then(|exit_status| {
                if exit_status.success() {
                    Ok(exit_status)
                } else {
                    Err(OllamaBuildpackError::OllamaPullExitStatus(exit_status))
                }
            })?;

        libherokubuildpack::log::log_header("Creating custom model");

        Command::new("ollama")
            .args([
                "create",
                "custom_model",
                "-f",
                &ollama_modelfile_path.to_string_lossy().to_string(),
            ])
            .envs(&env)
            .spawn()
            .map_err(OllamaBuildpackError::OllamaCreateIoError)?
            .wait()
            .map_err(OllamaBuildpackError::OllamaCreateIoError)
            .and_then(|exit_status| {
                if exit_status.success() {
                    Ok(exit_status)
                } else {
                    Err(OllamaBuildpackError::OllamaCreateExitStatus(exit_status))
                }
            })?;

        ollama_child_process
            .kill()
            .map_err(OllamaBuildpackError::GenericIoError)?;

        let process = ProcessBuilder::new(process_type!("web"), ["ollama", "serve"])
            .default(true)
            .build();

        BuildResultBuilder::new()
            .launch(LaunchBuilder::new().process(process).build())
            .build()
    }
}

buildpack_main!(OllamaBuildpack);
