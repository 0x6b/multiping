use std::{error::Error, net::IpAddr, time::Duration};

use clap::Parser;
use dns_lookup::lookup_host;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};
use tokio::time;

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Args {
    /// Ping targets
    #[arg(
        value_delimiter = ' ',
        default_value = "192.168.0.10 turingpi.local 192.168.0.31 192.168.0.32 192.168.0.33 192.168.0.34"
    )]
    targets: Vec<String>,
}

const TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Args { targets } = Args::parse();
    let client = Client::new(&Config::default())?;
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
            pb.set_prefix(format!("{ip}"));

            tasks.push(tokio::spawn(ping(client.clone(), ip, pb)));
        });

    for task in tasks {
        task.await?;
    }

    Ok(())
}

async fn ping(client: Client, addr: IpAddr, pb: ProgressBar) {
    let payload = [0; 56];
    let mut pinger = client.pinger(addr, PingIdentifier(0)).await;
    let duration = Duration::from_millis(1000);
    pinger.timeout(duration);

    let mut interval = time::interval(duration);

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
