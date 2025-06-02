use chrono::{DateTime, Local, TimeZone, Timelike, Utc};
use clap::Parser;
use std::{io::Error, mem::zeroed};

const NTP_MESSAGE_LENGTH: usize = 48;
const NTP_TO_UNIX_SECONDS: i64 = 2_208_988_800;
const LOCAL_ADDR: &'static str = "0.0.0.0:12300";

#[derive(Debug, Clone, Default, Copy)]
struct NTPTimestamp {
    seconds: u32,
    fraction: u32,
}

struct NTPMessage {
    data: [u8; NTP_MESSAGE_LENGTH],
}

#[derive(Debug)]
struct NTPResult {
    t1: DateTime<Utc>,
    t2: DateTime<Utc>,
    t3: DateTime<Utc>,
    t4: DateTime<Utc>,
}

impl NTPResult {
    fn offset(&self) -> i64 {
        let duration = (self.t2 - self.t1) + (self.t4 - self.t3);
        duration.num_milliseconds() / 2
    }
    fn delay(&self) -> i64 {
        let duration = (self.t4 - self.t1) - (self.t3 - self.t2);
        duration.num_milliseconds()
    }
}

impl From<NTPTimestamp> for DateTime<Utc> {
    fn from(value: NTPTimestamp) -> Self {
        let secs = value.seconds as i64 - NTP_TO_UNIX_SECONDS;
        let mut nanos = value.fraction as f64;
        nanos *= 1e9;
        nanos /= 2_f64.powi(32);
        Utc.timestamp(secs, nanos as u32)
    }
}

impl From<DateTime<Utc>> for NTPTimestamp {
    fn from(utc: DateTime<Utc>) -> Self {
        let secs = utc.timestamp() + NTP_TO_UNIX_SECONDS;
        let mut fraction = utc.nanosecond() as f64;
        fraction *= 2_f64.powi(32);
        fraction /= 1e9;
        NTPTimestamp {
            seconds: secs as u32,
            fraction: fraction as u32,
        }
    }
}
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
        let maybe_error = Error::last_os_error();
        let os_error_code = &maybe_error.raw_os_error();
        match os_error_code {
            Some(0) => {}
            Some(_) => eprintln!("Unable to set the time: {:?}", maybe_error),
            None => {}
        }
    }

    let now = Clock::get();
    match std {
        "timestamp" => println!("{}", now.timestamp()),
        "rfc2822" => println!("{}", now.to_rfc2822()),
        "rfc3339" => println!("{}", now.to_rfc3339()),
        _ => unreachable!(),
    }
}
