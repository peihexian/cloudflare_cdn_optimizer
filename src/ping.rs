use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::time::timeout;
use crate::config;

pub async fn ping(ip: IpAddr, timeout_duration: Duration) -> Option<Duration> {
    let start = Instant::now();
    let result = timeout(
        timeout_duration,
        Command::new("ping")
            .arg("-n")
            .arg("2")
            .arg("-w")
            .arg(timeout_duration.as_millis().to_string())
            .arg(ip.to_string())
            .output(),
    )
    .await;

    let timeelapsed = start.elapsed();

    if config::GLOBAL_CONFIG.optimization.debug {
        log::info!("ping ip {} result: {:?} ,take time {}ms", ip, result.as_ref().map(|r| r.is_ok()), timeelapsed.as_millis());
        //eprintln!("ping ip {} result: {:?} ,take time {}ms", ip, result.as_ref().map(|r| r.is_ok()), timeelapsed.as_millis());
    }

    match result {
        Ok(Ok(_)) => Some(timeelapsed),
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