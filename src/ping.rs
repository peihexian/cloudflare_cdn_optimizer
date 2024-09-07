use std::net::IpAddr;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use regex::Regex;
use encoding_rs::WINDOWS_1252;
use crate::config;

#[cfg(target_os = "windows")]
fn get_ping_command(ip: &IpAddr, timeout_ms: u64) -> Command {
    let mut cmd = Command::new("ping");
    cmd.arg("-n")
        .arg("1")
        .arg("-w")
        .arg(timeout_ms.to_string())
        .arg(ip.to_string());
    cmd
}

#[cfg(target_os = "linux")]
fn get_ping_command(ip: &IpAddr, timeout_ms: u64) -> Command {
    let mut cmd = Command::new("ping");
    cmd.arg("-c")
        .arg("1")
        .arg("-W")
        .arg((timeout_ms / 1000).to_string())
        .arg(ip.to_string());
    cmd
}

fn decode_output(output: &[u8]) -> String {
    let (cow, _, _) = WINDOWS_1252.decode(output);
    cow.into_owned()
}

fn parse_ping_output(output: &str) -> Option<Duration> {
    #[cfg(target_os = "windows")]
    let re = Regex::new(r"[=<](\d+)ms").unwrap();

    #[cfg(target_os = "linux")]
    let re = Regex::new(r"time=(\d+(?:\.\d+)?) ms").unwrap();

    let result = re.captures(output)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
        .map(|ms| Duration::from_micros((ms * 1000.0) as u64));

    if config::GLOBAL_CONFIG.optimization.debug {
        log::info!("Parsed ping output: {:?}", result);
    }

    result
}

pub async fn ping(ip: IpAddr, timeout_duration: Duration) -> Option<Duration> {
    let timeout_ms = timeout_duration.as_millis() as u64;
    let result = timeout(
        timeout_duration,
        get_ping_command(&ip, timeout_ms).output(),
    )
    .await;

    if config::GLOBAL_CONFIG.optimization.debug {
        log::info!("ping ip {} result: {:?}", ip, result.as_ref().map(|r| r.is_ok()));
    }

    match result {
        Ok(Ok(output)) => {
            let output_str = decode_output(&output.stdout);
            if config::GLOBAL_CONFIG.optimization.debug {
                log::info!("ping output for {}: {}", ip, output_str);
            }
            let duration = parse_ping_output(&output_str);
            if config::GLOBAL_CONFIG.optimization.debug {
                log::info!("Parsed duration for {}: {:?}", ip, duration);
            }
            duration
        },
        _ => None,
    }
}

pub async fn ping_ips(ips: Vec<IpAddr>, num_threads: usize) -> Vec<(IpAddr, Duration)> {
    use futures::stream::{self, StreamExt};

    stream::iter(ips)
        .map(|ip| async move {
            if let Some(duration) = ping(ip, Duration::from_secs(6)).await {
                Some((ip, duration))
            } else {
                None
            }
        })
        .buffer_unordered(num_threads)
        .filter_map(|result| async move { result })
        .collect()
        .await
}