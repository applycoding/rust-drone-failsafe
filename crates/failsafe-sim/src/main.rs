#![forbid(unsafe_code)]

use clap::Parser;
use failsafe_sim::SimRunner;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    scenario: PathBuf,
    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let toml = fs::read_to_string(&args.scenario)?;
    let mut sim = SimRunner::from_toml(&toml)?;
    let rows = sim.run_to_end();
    if let Some(out) = args.out {
        let mut wtr = csv::Writer::from_path(out)?;
        for row in rows {
            wtr.serialize(row)?;
        }
        wtr.flush()?;
    } else {
        for row in rows {
            println!("{} {:?} {:?}", row.t_ms, row.action, row.reason);
        }
    }
    Ok(())
}
