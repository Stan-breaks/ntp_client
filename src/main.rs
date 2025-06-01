use chrono::{DateTime, Local, TimeZone};
use clap::Parser;
use std::mem::zeroed;

/// command line interface for a ntp_client
#[derive(Parser, Debug)]
#[command(version,about,long_about=None)]
struct Args {
    /// Action command: "get" or "set"
    #[arg(long)]
    action: String,
    /// Format standard: "rfc2822", "rfc3339", or "timestamp"
    #[arg(short, long = "use-standard")]
    std: String,
    /// Datetime string (required only if action is "set")
    #[arg(long)]
    datetime: Option<String>,
}

struct Clock;
impl Clock {
    fn get() -> DateTime<Local> {
        Local::now()
    }
    #[cfg(not(windows))]
    fn set<Tz: TimeZone>(t: DateTime<Tz>) -> () {
        use libc::{settimeofday, timezone};
        use libc::{suseconds_t, time_t, timeval};

        let t = t.with_timezone(&Local);
        let mut u: timeval = unsafe { zeroed() };

        u.tv_sec = t.timestamp() as time_t;
        u.tv_usec = t.timestamp_subsec_micros() as suseconds_t;

        unsafe {
            let mock_tz: *const timezone = std::ptr::null();
            settimeofday(&u as *const timeval, mock_tz);
        }
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
        let t_ = args
            .datetime
            .as_deref()
            .expect("`--datetime` is required when using `--action set`");
        let parser = match std {
            "rfc2822" => DateTime::parse_from_rfc2822,
            "rfc3339" => DateTime::parse_from_rfc3339,
            _ => unimplemented!(),
        };
        let err_msg = format!("Unable to parse {} according to {}", t_, std);
        let t = parser(t_).expect(&err_msg);
        Clock::set(t);
    }

    let now = Clock::get();
    match std {
        "timestamp" => println!("{}", now.timestamp()),
        "rfc2822" => println!("{}", now.to_rfc2822()),
        "rfc3339" => println!("{}", now.to_rfc3339()),
        _ => unreachable!(),
    }
}
