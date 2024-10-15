mod args;
mod progress_style_map;

use std::{error::Error, net::IpAddr, sync::Arc, time::Duration};

use clap::Parser;
use dns_lookup::lookup_host;
use indicatif::{MultiProgress, ProgressBar};
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};

use crate::{args::Args, progress_style_map::PROGRESS_STYLE_MAP as styles};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Args { targets, interval, timeout } = Args::parse();

    let client = Arc::new(Client::new(&Config::default())?);
    let mut tasks = Vec::new();
    let m = MultiProgress::new();

    targets
        .into_iter()
        .filter_map(|s| {
            if let Ok(ip) = s.parse::<IpAddr>() {
                Some(ip)
            } else if let Ok(host) = lookup_host(&s) {
                host.into_iter()
                    .filter(|addr| addr.is_ipv4())
                    .collect::<Vec<_>>()
                    .first()
                    .cloned()
            } else {
                None
            }
        })
        .for_each(|ip| {
            let pb = m.add(ProgressBar::new(1));
            pb.set_style(styles.get("default"));
            pb.set_prefix(format!("{ip:15}"));
            tasks.push(tokio::spawn(ping(client.clone(), ip, interval, timeout, pb)));
        });

    for task in tasks {
        task.await?;
    }

    Ok(())
}

async fn ping(client: Arc<Client>, addr: IpAddr, interval: f64, timeout: f64, pb: ProgressBar) {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(0)).await;
    pinger.timeout(Duration::from_millis((timeout * 1000.0) as u64));
    let mut interval = tokio::time::interval(Duration::from_millis((interval * 1000.0) as u64));

    for idx in 0.. {
        interval.tick().await;

        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(packet), dur)) => {
                pb.set_style(styles.get("default"));
                pb.set_message(format!(
                    "{} bytes icmp_seq={} ttl={} time={dur:0.2?}",
                    packet.get_size(),
                    packet.get_sequence(),
                    packet.get_ttl().unwrap_or(u8::MAX),
                ));
                pb.inc(1);
            }
            Err(e) => {
                pb.set_style(styles.get("error"));
                pb.set_message(format!("{e}"));
                pb.inc(1);
            }
            _ => {}
        };
    }
}
