use async_std::task;
use futures::future::join_all;
use log::{info, warn};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;
use std::time::Duration;

mod caller;
use crate::caller::*;

mod lookout;
use crate::lookout::Lookout;

#[derive(Deserialize, Debug)]
struct Config {
    log: bool,
    error_delay: u64,
    alert_delay: u64,
    allowed_errors: u64,
    twilio: TwilioConfig,
    lookout: Vec<Lookout>,
}

// Padding width in logs
static PADDING: OnceCell<usize> = OnceCell::new();
static LOG: OnceCell<bool> = OnceCell::new();

fn info_warn(ctx: &str, msg: &str, warn: bool) {
    let mut name = format!("[{}] ", ctx).to_uppercase();
    for _ in 0..(PADDING
        .get()
        .expect("PADDING initialized before 'info' calls")
        .saturating_sub(ctx.len()))
    {
        name.push(' ');
    }
    if warn {
        warn!("{}{}", name, msg);
    } else {
        info!("{}{}", name, msg);
    }
}

fn info(ctx: &str, msg: &str) {
    info_warn(ctx, msg, false);
}

fn warn(ctx: &str, msg: &str) {
    info_warn(ctx, msg, true);
}

async fn start_lookout(lookout: Lookout, alert_delay: u64, error_delay: u64, allowed_errors: u64) {
    let mut errors = 0;
    while errors < allowed_errors {
        if lookout.scrape_timeout(alert_delay).await.is_err() {
            task::sleep(Duration::from_secs(error_delay)).await;
            errors += 1
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let path = match env::args().nth(1) {
        Some(p) => p,
        None => "lookout.toml".to_owned(),
    };
    // TODO better error message for read_to_string failure
    // (likely no config file to read from or incorrect path)
    let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
    PADDING
        .set(
            config
                .lookout
                .iter()
                .map(|l| l.name.len())
                .max()
                .expect("at least one lookout"),
        )
        .expect("set PADDING");
    LOG.set(config.log).expect("set LOG");
    set_twilio_config(config.twilio);

    let mut handles = vec![];
    for lookout in config.lookout {
        info(&lookout.name, "starting");
        handles.push(start_lookout(
            lookout,
            config.alert_delay,
            config.error_delay,
            config.allowed_errors,
        ));
    }
    join_all(handles).await;

    Ok(())
}
