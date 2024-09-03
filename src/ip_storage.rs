// src/ip_storage.rs
use std::fs::File;
use std::io::{BufWriter, Write};
use std::net::IpAddr;
use std::time::Duration;

pub fn save_top_ips(ips: &[(IpAddr, Duration)], filename: &str,top: usize) -> std::io::Result<()> {
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for (ip, duration) in ips.iter().take(top) {
        writeln!(writer, "{},{}", ip, duration.as_millis())?;
    }

    Ok(())
}