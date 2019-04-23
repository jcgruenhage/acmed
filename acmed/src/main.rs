use crate::main_event_loop::MainEventLoop;
use clap::{App, Arg};
use daemonize::Daemonize;
use log::{error, LevelFilter};

mod acme_proto;
mod certificate;
mod config;
mod error;
mod hooks;
mod keygen;
mod logs;
mod main_event_loop;
mod storage;

pub const APP_NAME: &str = "ACMEd";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_PID_FILE: &str = "/var/run/admed.pid";
pub const DEFAULT_CONFIG_FILE: &str = "/etc/acmed/acmed.toml";
pub const DEFAULT_ACCOUNTS_DIR: &str = "/etc/acmed/accounts";
pub const DEFAULT_CERT_DIR: &str = "/etc/acmed/certs";
pub const DEFAULT_CERT_FORMAT: &str = "{{name}}_{{algo}}.{{file_type}}.{{ext}}";
pub const DEFAULT_ALGO: &str = "rsa2048";
pub const DEFAULT_SLEEP_TIME: u64 = 3600;
pub const DEFAULT_POOL_TIME: u64 = 5000;
pub const DEFAULT_CERT_FILE_MODE: u32 = 0o644;
pub const DEFAULT_PK_FILE_MODE: u32 = 0o600;
pub const DEFAULT_ACCOUNT_FILE_MODE: u32 = 0o600;
pub const DEFAULT_KP_REUSE: bool = false;
pub const DEFAULT_LOG_SYSTEM: logs::LogSystem = logs::LogSystem::SysLog;
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Warn;
pub const DEFAULT_JWS_SIGN_ALGO: &str = "ES256";
pub const DEFAULT_POOL_NB_TRIES: usize = 10;
pub const DEFAULT_POOL_WAIT_SEC: u64 = 5;

fn main() {
    let matches = App::new(APP_NAME)
        .version(APP_VERSION)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Specify an alternative configuration file")
                .takes_value(true)
                .value_name("FILE"),
        )
        .arg(
            Arg::with_name("log-level")
                .long("log-level")
                .help("Specify the log level")
                .takes_value(true)
                .value_name("LEVEL")
                .possible_values(&["error", "warn", "info", "debug", "trace"]),
        )
        .arg(
            Arg::with_name("to-syslog")
                .long("log-syslog")
                .help("Sends log messages via syslog")
                .conflicts_with("to-stderr"),
        )
        .arg(
            Arg::with_name("to-stderr")
                .long("log-stderr")
                .help("Prints log messages to the standard error output")
                .conflicts_with("log-syslog"),
        )
        .arg(
            Arg::with_name("foregroung")
                .short("f")
                .long("foregroung")
                .help("Runs in the foregroung"),
        )
        .arg(
            Arg::with_name("pid-file")
                .long("pid-file")
                .help("Specifies the location of the PID file")
                .takes_value(true)
                .value_name("FILE")
                .conflicts_with("foregroung"),
        )
        .get_matches();

    match logs::set_log_system(
        matches.value_of("log-level"),
        matches.is_present("log-syslog"),
        matches.is_present("to-stderr"),
    ) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(2);
        }
    };

    if !matches.is_present("foregroung") {
        let pid_file = matches.value_of("pid-file").unwrap_or(DEFAULT_PID_FILE);
        let daemonize = Daemonize::new().pid_file(pid_file);
        match daemonize.start() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(3);
            }
        }
    }

    let config_file = matches.value_of("config").unwrap_or(DEFAULT_CONFIG_FILE);
    let mut srv = match MainEventLoop::new(&config_file) {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    srv.run();
}
