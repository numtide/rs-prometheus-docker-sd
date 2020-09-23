#[macro_use]
extern crate log;

mod types;

use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::models::ContainerSummary;
use bollard::Docker;

use env_logger;
use std::collections::HashMap;
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use std::{thread, time};
use tempdir::TempDir;

use tokio::runtime::Runtime;

use types::{
    get_config_docker_compose_service, get_config_hostname, get_config_job,
    get_config_metrics_path, get_config_port, get_config_scheme, get_container_hostname,
    get_container_name, get_scrape_enabled, PromConfig,
};

// Maybe_add_docker_info
//
// Add the docker information to the prometheus struct, if it has the right label.

// Async function to take running docker's information
// , and turn into promotheus' json format
async fn maybe_add_container_info<'a>(
    docker: &'a Docker,
    mut pconfig: PromConfig,
    csummary: &'a ContainerSummary,
) -> Result<PromConfig, Box<dyn std::error::Error + 'static>> {
    let container = docker
        .inspect_container(
            csummary.id.as_ref().unwrap(),
            None::<InspectContainerOptions>,
        )
        .await
        .unwrap();

    let empty_hash = HashMap::new();
    let docker_labels = match csummary.clone().labels {
        Some(x) => x,
        _ => empty_hash,
    };
    // let docker_labels = get_config_labels(container_config);
    let container_name = get_container_name(container.clone());

    pconfig.labels.job = container_name.clone();
    pconfig.labels.name = container_name.clone();
    pconfig.labels.id = get_container_hostname(container.clone());

    if !docker_labels.is_empty() {
        match get_scrape_enabled(&docker_labels).unwrap_or(false) {
            true => {
                let job_name = get_config_job(docker_labels.clone());
                debug!("Container {} is enabled for prometheus.", container_name);
                if let true = !job_name.is_empty() {
                    pconfig.labels.job = job_name.clone();
                    debug!("Set job name to {}.", job_name)
                }
                debug!("Job name is not set, using default value.")
            }
            false => {
                debug!(
                    "Container {} has no \"prometheus-scrape.enabled\" label and is ignored.",
                    container_name
                );
                return Ok(PromConfig::new());
            }
        }
    } else {
        error!("Docker doesn't have labels")
    }

    let port = get_config_port(docker_labels.clone());
    let hostname = get_config_hostname(docker_labels.clone(), container_name.clone());
    let target = format!("{}:{}", hostname, port);

    pconfig.targets.push(target);
    pconfig.labels.scheme = get_config_scheme(docker_labels.clone());
    pconfig.labels.metrics_path = get_config_metrics_path(docker_labels.clone());
    pconfig.labels.com_docker_compose_service =
        get_config_docker_compose_service(docker_labels.clone());

    if pconfig.targets.len() > 0 {
        Ok(pconfig)
    } else {
        Ok(PromConfig::new())
    }
}

async fn run(refresh_interval_sec: Duration) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    let docker = Docker::connect_with_unix_defaults().unwrap();
    #[cfg(windows)]
    let docker = Docker::connect_with_named_pipe_defaults().unwrap();
    let mut previous_config = String::new();

    loop {
        // TODO: Create the empty struct
        let mut promconfig: Vec<PromConfig> = Vec::new();
        let pconfig = PromConfig::new();

        // Get the list of containers
        let mut list_container_filters = HashMap::new();
        list_container_filters.insert("status", vec!["running"]);
        let containers = &docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: list_container_filters,
                ..Default::default()
            }))
            .await?;

        // Query each container and add its info to the config if it has the right label
        for container in containers {
            let result = maybe_add_container_info(&docker, pconfig.clone(), container).await?;
            promconfig.push(result)
        }

        promconfig.retain(|e| !e.targets.is_empty());

        if promconfig.len() < 1 {
            error!("No containers have label \"prometheus-scrape.enabled\" set to true")
        }

        // Only write if the content has changed
        let folder = Path::new("/promotheus-docker-sd");
        let config_path = folder.join("docker-targets.json");
        if !folder.exists() {
            println!("Folder doesn't exist, creating a new folder...");
            if let Err(err) = fs::create_dir_all(folder) {
                error!("Cannot create {:?} due to {} error", folder, err)
            }
            println!("Folder '/promotheus-docker-sd/' created.");
            println!("Creating a new 'docker-targets.json' file");
            if let Err(err) = File::create(config_path.clone()) {
                error!("Error: Cannot create config file due to: {}", err)
            }
            println!("File 'docker-targets.json' created.");
        }

        let current_config = serde_json::to_string(&promconfig)?;

        let tmp = Path::new("/tmp");
        if !tmp.exists() {
            println!("'/tmp' doesn't exist, creating a new tmp folder...");
            if let Err(err) = fs::create_dir_all(tmp) {
                error!("Cannot create {:?} due to {} error", tmp, err)
            }
            println!("Folder '/tmp' created.");
        }
        let tmp_dir = TempDir::new_in("/tmp/", "promotheus-docker-sd")?;
        let tmp_path = tmp_dir.path().join("docker-targets.json");
        if let Err(err) = fs::write(tmp_path.clone(), current_config.clone()) {
            error!("Cannot write to temp file due to: {}", err)
        }

        if current_config != previous_config {
            if previous_config.is_empty() {
                println!("Creating 'docker-targets.json'...");
            } else {
                println!("Found new config. Attempting to update 'docker-targets.json'...");
            }
            if let Err(err) = fs::copy(tmp_path, config_path) {
                error!("Cannot copy to 'docker-targets.json' due to: {}", err)
            }
            previous_config = current_config;

            if let Err(err) = fs::remove_dir_all(tmp_dir) {
                error!("Cannot delete temporary directory and files: {}", err)
            }
        }

        for pc in promconfig {
            println!("{:#?}", pc)
        }

        // Wait for a bit
        thread::sleep(refresh_interval_sec);
    }
}

fn main() {
    env_logger::init();

    let refresh_interval_sec = time::Duration::from_secs(900);

    let mut rt = Runtime::new().unwrap();

    rt.block_on(run(refresh_interval_sec)).unwrap();
}
