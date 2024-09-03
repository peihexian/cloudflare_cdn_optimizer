// src/ip_utils.rs
use ipnetwork::IpNetwork;
use std::net::IpAddr;
use std::str::FromStr;

pub fn parse_cidr_list(cidr_list: &Vec<String>) -> Vec<IpAddr> {
    cidr_list
        .iter()
        .flat_map(|cidr| {
            IpNetwork::from_str(cidr.trim())
                .map(|network| network.iter().collect::<Vec<IpAddr>>())
                .unwrap_or_else(|_| {
                    eprintln!("Invalid CIDR: {}", cidr);
                    vec![]
                })
        })
        .collect()
}