use chrono::{DateTime, Local};
use clap::Parser;

/// command line interface for a ntp_client
#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]
struct Args {
    ///action command
    #[arg(long)]
    action: String,
    ///std command
    #[arg(short, long = "use-standard")]
    std: String,
    ///datetime command
    #[arg(long)]
    datetime: String,
}

struct Clock;
impl Clock {
    fn get() -> DateTime<Local> {
        Local::now()
    }
    fn set() -> ! {
        unimplemented!()
    }
}

fn main() {
    let args = Args::parse();
    let action = match args.action.as_str() {
        "set" => "set",
        _ => "get",
    };
    let std = args.std.as_str();
    if action == "set" {
        unimplemented!()
    }

    let now = Clock::get();
    match std {
        "timestamp" => println!("{}", now.timestamp()),
        "rfc2822" => println!("{}", now.to_rfc2822()),
        "rfc3339" => println!("{}", now.to_rfc3339()),
        _ => unreachable!(),
    }
}
