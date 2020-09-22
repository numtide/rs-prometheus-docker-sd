use serde::{Deserialize, Serialize};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::default::Default;

use bollard::models::ContainerInspectResponse;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromConfigLabel {
    pub job: String,
    pub name: String,
    pub id: String,
    pub scheme: String,
    pub metrics_path: String,
    pub com_docker_compose_service: String,
}

impl Default for PromConfigLabel {
    fn default() -> Self {
        PromConfigLabel {
            job: String::from(""),
            name: String::from(""),
            id: String::from(""),
            scheme: String::from(""),
            metrics_path: String::from(""),
            com_docker_compose_service: String::from(""),
        }
    }
}

impl PromConfigLabel {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PromConfig {
    // labels for container information
    pub labels: PromConfigLabel,
    pub targets: Vec<String>,
}

impl PromConfig {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for PromConfig {
    fn default() -> Self {
        PromConfig {
            labels: PromConfigLabel::new(),
            targets: Vec::new(),
        }
    }
}

pub fn get_scrape_enabled(hash: &HashMap<String, String, RandomState>) -> Option<bool> {
    let scrape = &"prometheus-scrape.enabled".to_string();
    let enabled_label = hash.get(scrape);
    enabled_label.map(|e| e.eq(&String::from("true")))
}

pub fn get_config_job(hash: HashMap<String, String, RandomState>) -> String {
    let config_job = &"prometheus-scrape.job_name".to_string();
    if let Some(job) = hash.get(config_job) {
        return String::from(job);
    }
    return String::from("");
}

pub fn get_config_port(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("prometheus-scrape.port");
    let port = String::from("9090");
    if let Some(new_port) = hash.get(config_port) {
        debug!("Port is set to {}.", new_port);
        return String::from(new_port);
    }
    debug!("Job name is not set, using default value.");
    return port;
}

pub fn get_config_scheme(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("prometheus-scrape.scheme");
    let scheme = String::from("http");
    if let Some(new_scheme) = hash.get(config_port) {
        debug!("Port is set to {}.", new_scheme);
        return String::from(new_scheme);
    }
    debug!("Job name is not set, using default value.");
    return scheme;
}

pub fn get_config_metrics_path(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("prometheus-scrape.metrics_path");
    let metrics_path = String::from("/metrics");
    if let Some(new_path) = hash.get(config_port) {
        debug!("Port is set to {}.", new_path);
        return String::from(new_path);
    }
    debug!("Job name is not set, using default value.");
    return String::from(metrics_path);
}

pub fn get_config_docker_compose_service(hash: HashMap<String, String, RandomState>) -> String {
    let config_port = &String::from("com.docker.compose.service");
    // TODO: Find out what is compose_service's default, given that it is not in documentation
    let compose_service = String::from("");
    if let Some(new_compose_service) = hash.get(config_port) {
        debug!("Compose service name is set to {}.", new_compose_service);
        return String::from(new_compose_service);
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
pub fn get_config_hostname(hash: HashMap<String, String, RandomState>, cname: String) -> String {
    let config_hostname = &String::from("prometheus-scrape.hostname");
    let config_ip_hostname = &String::from("prometheus-scrape.ip_as_hostname");
    if let Some(new_hostname) = hash.get(config_hostname) {
        debug!("Hostname is set to {}.", new_hostname);
        return String::from(new_hostname);
    }
    if let Some(new_ip_hostname) = hash.get(config_ip_hostname) {
        match new_ip_hostname.eq(&String::from("true")) {
            true => {
                debug!("IP address for hostname is set to {}.", new_ip_hostname);
                return String::from(new_ip_hostname);
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

pub fn get_container_name(ctr: ContainerInspectResponse) -> String {
    match ctr.name {
        Some(x) => x
            .strip_prefix("/")
            .map(|e| String::from(e))
            .unwrap_or(String::from("")),
        _ => String::from(""),
    }
}

pub fn get_container_hostname(ctr: ContainerInspectResponse) -> String {
    match ctr.config.and_then(|e| e.hostname) {
        Some(x) => return x,
        _ => String::from(""),
    }
}
