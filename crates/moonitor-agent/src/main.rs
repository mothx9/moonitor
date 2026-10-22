use anyhow::{Context, Result, bail};
use moonitor_agent::Collector;
use std::{
    io::{self, Write},
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    let mut id = "local".to_owned();
    let mut interval = 1000u64;
    let mut count = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--node-id" => id = args.next().context("Missing node ID")?,
            "--interval-ms" => interval = args.next().context("Missing interval")?.parse()?,
            "--count" => count = Some(args.next().context("Missing count")?.parse::<usize>()?),
            "--help" | "-h" => {
                println!(
                    "moonitor-agent [--node-id ID] [--interval-ms 1000] [--count N]\nStreams read-only JSON telemetry to stdout; no listener or service is installed."
                );
                return Ok(());
            }
            _ => bail!("Unknown argument: {arg}"),
        }
    }
    if !(500..=60_000).contains(&interval) {
        bail!("Interval must be between 500 and 60000 ms");
    }
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
    {
        bail!("Invalid node ID");
    }
    let mut collector = Collector::new(id);
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.max(Duration::from_millis(250)));
    let mut output = io::BufWriter::new(io::stdout().lock());
    let mut sent = 0;
    while count.is_none_or(|n| sent < n) {
        let start = Instant::now();
        let snapshot = collector.sample();
        let json = serde_json::to_vec(&snapshot)?;
        if json.len() > moonitor_core::MAX_MESSAGE_BYTES {
            bail!("Telemetry exceeds message limit");
        }
        if let Err(error) = output
            .write_all(&json)
            .and_then(|_| output.write_all(b"\n"))
            .and_then(|_| output.flush())
        {
            if error.kind() == io::ErrorKind::BrokenPipe {
                break;
            }
            return Err(error.into());
        }
        sent += 1;
        if count.is_none_or(|n| sent < n) {
            std::thread::sleep(Duration::from_millis(interval).saturating_sub(start.elapsed()));
        }
    }
    Ok(())
}
