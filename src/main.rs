#[macro_use]
extern crate log;

mod types;

use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::models::ContainerSummary;
use bollard::Docker;

use env_logger;
use std::collections::HashMap;
use std::default::Default;

use futures_util::stream;
use futures_util::stream::StreamExt;
use tokio::runtime::Runtime;

use types::{
    get_config_docker_compose_service, get_config_hostname, get_config_job,
    get_config_metrics_path, get_config_port, get_config_scheme, get_container_hostname,
    get_container_name, get_scrape_enabled, PContainerInfo,
};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    let docker = Docker::connect_with_unix_defaults().unwrap();
    #[cfg(windows)]
    let docker = Docker::connect_with_named_pipe_defaults().unwrap();

    let mut list_container_filters = HashMap::new();
    list_container_filters.insert("status", vec!["running"]);

    let containers = &docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: list_container_filters,
            ..Default::default()
        }))
        .await?;

    // create multiple stream
    let docker_stream = stream::repeat(docker);

    // zip all the stream with containers' result, then apply each of the containers
    // with `docker_json_to_promotheus` function
    docker_stream
        .zip(stream::iter(containers))
        .for_each_concurrent(2, docker_json_to_promotheus)
        .await;
    Ok(())
}

// Async function to take running docker's information
// , and turn into promotheus' json format
async fn docker_json_to_promotheus(arg: (Docker, &ContainerSummary)) -> () {
    let (docker, csummary) = arg;
    let container = docker
        .inspect_container(
            csummary.id.as_ref().unwrap(),
            None::<InspectContainerOptions>,
        )
        .await
        .unwrap();

    let empty_hash = HashMap::new();
    let docker_labels = match csummary.labels.as_ref() {
        Some(x) => x,
        _ => &empty_hash,
    };
    // let docker_labels = get_config_labels(container_config);
    let container_name = get_container_name(&container);
    let mut pcontainer = PContainerInfo::new();

    pcontainer.labels.job = container_name;
    pcontainer.labels.name = container_name;
    pcontainer.labels.id = get_container_hostname(&container);

    if !docker_labels.is_empty() {
        match get_scrape_enabled(&docker_labels).unwrap_or(false) {
            true => {
                let job_name = get_config_job(&docker_labels);
                debug!("Container {} is enabled for prometheus.", container_name);
                if let true = !job_name.is_empty() {
                    pcontainer.labels.job = job_name;
                    debug!("Set job name to {}.", job_name)
                }
                debug!("Job name is not set, using default value.")
            }
            false => debug!(
                "Container {} has no \"prometheus-scrape.enabled\" label and is ignored.",
                container_name
            ),
        }
    } else {
        error!("Docker doesn't have labels")
    }

    let port = get_config_port(&docker_labels);
    let hostname = get_config_hostname(&docker_labels, container_name);
    let target = format!("{}:{}", hostname, port);

    pcontainer.targets.push(target);
    pcontainer.labels.scheme = get_config_scheme(&docker_labels);
    pcontainer.labels.metrics_path = get_config_metrics_path(&docker_labels);
    pcontainer.labels.com_docker_compose_service =
        get_config_docker_compose_service(&docker_labels);
    println!("{:#?}", pcontainer)
}

fn main() {
    env_logger::init();

    let mut rt = Runtime::new().unwrap();

    rt.block_on(run()).unwrap();
}
