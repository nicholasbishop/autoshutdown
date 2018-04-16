extern crate regex;
extern crate simple_error;
#[macro_use]
extern crate structopt;

use regex::Regex;
use simple_error::SimpleError;
use std::error::Error;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use structopt::StructOpt;

fn parse_duration(input: &str) -> Result<Duration, Box<Error>> {
    let re = Regex::new(r"\s*(\d+)(\w)\s*")?;
    if let Some(captures) = re.captures(input) {
        let num = captures[1].parse::<u64>()?;
        match &captures[2] {
            "H" | "h" => Ok(Duration::from_secs(num * 60 * 60)),
            "M" | "m" => Ok(Duration::from_secs(num * 60)),
            "S" | "s" => Ok(Duration::from_secs(num)),
            _ => Err(SimpleError::new("failed to parse duration").into()),
        }
    } else {
        Err(SimpleError::new("failed to parse duration").into())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Automatically shut down after a period of inactivity")]
struct Config {
    #[structopt(long = "heartbeat-path", default_value = "/run/last_heartbeat",
                parse(from_os_str))]
    heartbeat_path: PathBuf,

    #[structopt(long = "check-interval", default_value = "1m",
                parse(try_from_str = "parse_duration"))]
    check_interval: Duration,

    #[structopt(long = "grace-duration", default_value = "5m",
                parse(try_from_str = "parse_duration"))]
    grace_duration: Duration,

    #[structopt(long = "shutdown-command", default_value = "poweroff")]
    shutdown_command: String,
}

fn read_last_heartbeat(config: &Config) -> Result<SystemTime, Box<Error>> {
    let metadata = fs::metadata(&config.heartbeat_path)?;
    return Ok(metadata.modified()?)
}

fn initialize_heartbeat(config: &Config) -> Result<(), Box<Error>> {
    fs::File::create(&config.heartbeat_path)?;
    Ok(())
}

fn parse_command(input: &String) -> Option<Command> {
    let mut parts = input.split_whitespace();
    if let Some(first) = parts.next() {
        let mut command = Command::new(first);
        command.args(parts);
        Some(command)
    } else {
        None
    }
}

fn shutdown(config: &Config) {
    if let Some(mut command) = parse_command(&config.shutdown_command) {
        command.exec();
    } else {
        eprintln!("failed to parse command");
    }
}

fn check_heartbeat(config: &Config) {
    match read_last_heartbeat(&config) {
        Ok(heartbeat) => {
            let limit = heartbeat + config.grace_duration;

            if SystemTime::now() > limit {
                shutdown(config);
            }
        }
        Err(err) => {
            eprintln!("failed to read heartbeat: {}", err);

            if let Err(init_err) = initialize_heartbeat(&config) {
                eprintln!("failed to init heartbeat: {}", init_err);
            }
        }
    }
}

fn main() {
    let config = Config::from_args();

    loop {
        check_heartbeat(&config);
        sleep(config.check_interval);
    }
}
