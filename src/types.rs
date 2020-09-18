use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::default::Default;

use bollard::models::ContainerInspectResponse;

#[derive(Debug)]
pub struct PContainerLabel<'a> {
    pub job: &'a str,
    pub name: &'a str,
    pub id: &'a str,
    pub scheme: &'a str,
    pub metrics_path: &'a str,
    pub com_docker_compose_service: &'a str,
}

impl<'a> Default for PContainerLabel<'a> {
    fn default() -> Self {
        PContainerLabel {
            job: "",
            name: "",
            id: "",
            scheme: "",
            metrics_path: "",
            com_docker_compose_service: "",
        }
    }
}

impl<'a> PContainerLabel<'a> {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug)]
pub struct PContainerInfo<'a> {
    // labels for container information
    pub labels: PContainerLabel<'a>,
    pub targets: Vec<String>,
}

impl<'a> PContainerInfo<'a> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> Default for PContainerInfo<'a> {
    fn default() -> Self {
        PContainerInfo {
            labels: PContainerLabel::new(),
            targets: Vec::new(),
        }
    }
}

pub fn get_scrape_enabled(hash: &HashMap<String, String, RandomState>) -> Option<bool> {
    let scrape = &"prometheus-scrape.enabled".to_string();
    let enabled_label = hash.get(scrape);
    enabled_label.map(|e| e.eq(&String::from("true")))
}

pub fn get_config_job(hash: &HashMap<String, String, RandomState>) -> &str {
    let config_job = &"prometheus-scrape.job_name".to_string();
    if let Some(job) = hash.get(config_job) {
        return job;
    }
    return "";
}

pub fn get_config_port(hash: &HashMap<String, String, RandomState>) -> &str {
    let config_port = &String::from("prometheus-scrape.port");
    let port = "9090";
    if let Some(new_port) = hash.get(config_port) {
        debug!("Port is set to {}.", new_port);
        return new_port;
    }
    debug!("Job name is not set, using default value.");
    return port;
}

pub fn get_config_scheme(hash: &HashMap<String, String, RandomState>) -> &str {
    let config_port = &String::from("prometheus-scrape.scheme");
    let scheme = "http";
    if let Some(new_scheme) = hash.get(config_port) {
        debug!("Port is set to {}.", new_scheme);
        return new_scheme;
    }
    debug!("Job name is not set, using default value.");
    return scheme;
}

pub fn get_config_metrics_path(hash: &HashMap<String, String, RandomState>) -> &str {
    let config_port = &String::from("prometheus-scrape.metrics_path");
    let metrics_path = "/metrics";
    if let Some(new_path) = hash.get(config_port) {
        debug!("Port is set to {}.", new_path);
        return new_path;
    }
    debug!("Job name is not set, using default value.");
    return metrics_path;
}

pub fn get_config_docker_compose_service(hash: &HashMap<String, String, RandomState>) -> &str {
    let config_port = &String::from("com.docker.compose.service");
    // TODO: Find out what is compose_service's default, given that it is not in documentation
    let compose_service = "";
    if let Some(new_compose_service) = hash.get(config_port) {
        debug!("Compose service name is set to {}.", new_compose_service);
        return new_compose_service;
    }
    debug!("Job name is not set, using default value.");
    return compose_service;
}

// Get hostname or ip address set in the docker's label.
// There are two option's label:
// 1. `prometheus-scrape.hostname`
// 2. `prometheus-scrape.ip_as_hostname`
// `promotheus-scrape.hostname` is preferred first, and then `prometheus-scrape.ip_as_hostname`
// in case both are not detected, the default container's name will be used.
pub fn get_config_hostname<'a>(
    hash: &'a HashMap<String, String, RandomState>,
    cname: &'a str,
) -> &'a str {
    let config_hostname = &String::from("prometheus-scrape.hostname");
    let config_ip_hostname = &String::from("prometheus-scrape.ip_as_hostname");
    if let Some(new_hostname) = &hash.get(config_hostname) {
        debug!("Hostname is set to {}.", new_hostname);
        return new_hostname;
    }
    if let Some(new_ip_hostname) = hash.get(config_ip_hostname) {
        match new_ip_hostname.eq(&String::from("true")) {
            true => {
                debug!("IP address for hostname is set to {}.", new_ip_hostname);
                return &new_ip_hostname;
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

pub fn get_container_name(ctr: &ContainerInspectResponse) -> &str {
    match &ctr.name {
        Some(x) => x.strip_prefix("/").unwrap_or(""),
        _ => "",
    }
}

pub fn get_container_hostname(ctr: &ContainerInspectResponse) -> &str {
    match ctr.config.as_ref().and_then(|e| e.hostname.as_ref()) {
        Some(x) => return &x,
        _ => "",
    }
}
