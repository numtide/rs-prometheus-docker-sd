#[macro_use]
extern crate log;

use bollard::container::{InspectContainerOptions, ListContainersOptions};
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use bollard::Docker;

use env_logger;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::default::Default;

use futures_util::stream;
use futures_util::stream::StreamExt;
use tokio::runtime::Runtime;

#[derive(Debug)]
struct PContainerLabel {
    job: String,
    name: String,
    id: String,
    scheme: String,
    metrics_path: String,
    com_docker_compose_service: String,
}

impl Default for PContainerLabel {
    fn default() -> Self {
        PContainerLabel {
            job: String::new(),
            name: String::new(),
            id: String::new(),
            scheme: String::new(),
            metrics_path: String::new(),
            com_docker_compose_service: String::new(),
        }
    }
}

impl PContainerLabel {
    fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
struct PContainerInfo {
    // labels for container information
    labels: PContainerLabel,
    targets: Vec<String>,
}

impl PContainerInfo {
    fn new() -> Self {
        Default::default()
    }
}

impl Default for PContainerInfo {
    fn default() -> Self {
        PContainerInfo {
            labels: PContainerLabel::new(),
            targets: Vec::new(),
        }
    }
}

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

    let docker_stream = stream::repeat(docker);
    docker_stream
        .zip(stream::iter(containers))
        .for_each_concurrent(2, docker_json_to_promotheus)
        .await;
    Ok(())
}

async fn docker_json_to_promotheus(arg: (Docker, &ContainerSummary)) -> () {
    let (docker, csummary) = arg;
    let container = docker
        .inspect_container(
            csummary.id.as_ref().unwrap(),
            None::<InspectContainerOptions>,
        )
        .await
        .unwrap();

    let container_name = get_container_name(container.clone());
    let mut pcontainer = PContainerInfo::new();

    pcontainer.labels.job = container_name.clone();
    pcontainer.labels.name = container_name.clone();
    pcontainer.labels.id = get_container_hostname(container.clone());

    let docker_labels = match container.clone().config.and_then(|e| e.labels) {
        Some(x) => x,
        _ => HashMap::new(),
    };

    if !docker_labels.is_empty() {
        match get_scrape_enabled(docker_labels.clone()).unwrap_or(false) {
            true => {
                let job_name = get_config_job(docker_labels.clone());
                debug!("Container {} is enabled for prometheus.", container_name);
                if let true = !job_name.is_empty() {
                    pcontainer.labels.job = job_name.clone();
                    debug!("Set job name to {}.", job_name.clone())
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

    let port = get_config_port(docker_labels.clone());
    let hostname = get_config_hostname(docker_labels.clone(), container_name.clone());
    let target = format!("{}:{}", hostname, port);

    pcontainer.targets.push(target);
    pcontainer.labels.scheme = get_config_scheme(docker_labels.clone());
    pcontainer.labels.metrics_path = get_config_metrics_path(docker_labels.clone());
    pcontainer.labels.com_docker_compose_service =
        get_config_docker_compose_service(docker_labels.clone());
    println!("{:#?}", pcontainer)
}

fn main() {
    env_logger::init();

    let mut rt = Runtime::new().unwrap();

    rt.block_on(run()).unwrap();
}

fn get_scrape_enabled(hash: HashMap<String, String, RandomState>) -> Option<bool> {
    let scrape = &"prometheus-scrape.enabled".to_string();
    let enabled_label = hash.get(scrape);
    enabled_label.map(|e| e.eq(&String::from("true")))
}

fn get_config_job(hash: HashMap<String, String, RandomState>) -> String {
    let config_job = &"prometheus-scrape.job_name".to_string();
    hash.get(config_job)
        .map(|e| e.to_string())
        .unwrap_or(String::from(""))
}

fn get_config_port(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("prometheus-scrape.port");
    let port = String::from("9090");
    if let Some(new_port) = hash.get(config_port).map(|e| e.to_string()) {
        debug!("Port is set to {}.", new_port);
        return new_port;
    }
    debug!("Job name is not set, using default value.");
    return port;
}

fn get_config_scheme(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("prometheus-scrape.scheme");
    let scheme = String::from("http");
    if let Some(new_scheme) = hash.get(config_port).map(|e| e.to_string()) {
        debug!("Port is set to {}.", new_scheme);
        return new_scheme;
    }
    debug!("Job name is not set, using default value.");
    return scheme;
}

fn get_config_metrics_path(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("prometheus-scrape.metrics_path");
    let metrics_path = String::from("/metrics");
    if let Some(new_path) = hash.get(config_port).map(|e| e.to_string()) {
        debug!("Port is set to {}.", new_path);
        return new_path;
    }
    debug!("Job name is not set, using default value.");
    return metrics_path;
}

fn get_config_docker_compose_service(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("com.docker.compose.service");
    // TODO: Find out what is compose_service's default, given that it is not in documentation
    let compose_service = String::from("");
    if let Some(new_compose_service) = hash.get(config_port).map(|e| e.to_string()) {
        debug!("Compose service name is set to {}.", new_compose_service);
        return new_compose_service;
    }
    debug!("Job name is not set, using default value.");
    return compose_service;
}

fn get_config_hostname(hash: HashMap<String, String, RandomState>, cname: String) -> String {
    let config_hostname = &String::from("prometheus-scrape.hostname");
    let config_ip_hostname = &String::from("prometheus-scrape.ip_as_hostname");
    if let Some(new_hostname) = hash.get(config_hostname) {
        debug!("Hostname is set to {}.", new_hostname);
        return new_hostname.to_string();
    }
    if let Some(new_ip_hostname) = hash.get(config_ip_hostname).map(|e| e.to_string()) {
        match new_ip_hostname.eq(&String::from("true")) {
            true => {
                debug!("IP address for hostname is set to {}.", new_ip_hostname);
                return new_ip_hostname;
            }
            false => {
                debug!("hostname is not set, using default value.");
                return cname;
            }
        }
    }

    debug!("hostname is not set, using default value.");
    return cname;
}

fn get_container_name(ctr: ContainerInspectResponse) -> String {
    match ctr.name {
        Some(x) => x
            .strip_prefix("/")
            .map(|n| n.to_string())
            .unwrap_or(String::from("")),
        _ => String::from(""),
    }
}

fn get_container_hostname(ctr: ContainerInspectResponse) -> String {
    ctr.config
        .and_then(|e| e.hostname)
        .unwrap_or(String::from(""))
}
