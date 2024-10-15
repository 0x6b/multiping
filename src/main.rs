use std::{error::Error, net::IpAddr, sync::Arc, time::Duration};

use clap::Parser;
use dns_lookup::lookup_host;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};
use tokio::{spawn, time};

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// Ping targets
    #[arg(
        value_delimiter = ' ',
        default_value = "192.168.0.10 turingpi.local 192.168.0.31 192.168.0.32 192.168.0.33 192.168.0.34"
    )]
    targets: Vec<String>,

    /// ping interval in seconds
    #[arg(short, long, default_value = "1")]
    interval: f64,

    /// ping timeout in seconds
    #[arg(short, long, default_value = "1")]
    timeout: f64,
}

const TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

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
            pb.set_style(
                ProgressStyle::with_template("{spinner:.bold} {prefix}: {wide_msg}")
                    .unwrap()
                    .tick_chars(TICK_CHARS),
            );
            pb.set_prefix(format!("{ip:15}"));

            tasks.push(spawn(ping(client.clone(), ip, interval, timeout, pb)));
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
    let mut interval = time::interval(Duration::from_millis((interval * 1000.0) as u64));

    for idx in 0.. {
        interval.tick().await;

        match pinger.ping(PingSequence(idx), &payload).await {
            Ok((IcmpPacket::V4(packet), dur)) => {
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.bold} {prefix:.green}: {wide_msg}")
                        .unwrap()
                        .tick_chars(TICK_CHARS),
                );
                pb.set_message(format!(
                    "{} bytes icmp_seq={} ttl={} time={:0.2?}",
                    packet.get_size(),
                    packet.get_sequence(),
                    packet.get_ttl().unwrap_or(u8::MAX),
                    dur
                ));
                pb.inc(1);
            }
            Err(e) => {
                pb.set_style(
                    ProgressStyle::with_template("{spinner:.bold} {prefix:.red}: {wide_msg}")
                        .unwrap()
                        .tick_chars(TICK_CHARS),
                );
                pb.set_message(format!("{e}"));
                pb.inc(1);
            }
            _ => {}
        };
    }
}
