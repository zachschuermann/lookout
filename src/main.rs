use futures::future::join_all;
use log::{info, warn};
use once_cell::sync::OnceCell;
use serde::Deserialize;

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

async fn start_lookout(lookout: Lookout) {
    loop {
        lookout.scrape_timeout().await.expect("scrape_timeout");
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config: Config = toml::from_str(&std::fs::read_to_string("lookout.toml")?)?;
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
        handles.push(start_lookout(lookout));
    }
    join_all(handles).await;

    Ok(())
}
