use std::{error::Error, net::IpAddr, time::Duration};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use surge_ping::{Client, Config, IcmpPacket, PingIdentifier, PingSequence};
use tokio::time;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    /// IP addresses to ping
    #[structopt(
        value_delimiter = " ",
        default_value = "192.168.0.30 192.168.0.31 192.168.0.32 192.168.0.33 192.168.0.34"
    )]
    ip_addresses: Vec<String>,
}

static TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Args { ip_addresses } = Args::from_args();
    let client = Client::new(&Config::default())?;
    let mut tasks = Vec::new();
    let m = MultiProgress::new();

    ip_addresses
        .into_iter()
        .filter_map(|s| s.parse::<IpAddr>().ok())
        .for_each(|ip| {
            let pb = m.add(ProgressBar::new(1));
            pb.set_style(
                ProgressStyle::with_template("{spinner} {prefix} {wide_msg}")
                    .unwrap()
                    .tick_chars(TICK_CHARS),
            );
            pb.set_prefix(format!("{}:", ip));

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
                    ProgressStyle::with_template("{spinner} {prefix:.green} {wide_msg}")
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
                    ProgressStyle::with_template("{spinner} {prefix:.red} {wide_msg}")
                        .unwrap()
                        .tick_chars(TICK_CHARS),
                );
                pb.set_message(format!("{}", e));
                pb.inc(1);
            }
            _ => {}
        };
    }
}
