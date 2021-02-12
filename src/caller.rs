/// Exposes calling/texting functionality via Twilio

use once_cell::sync::Lazy;
use twilio::{OutboundCall, OutboundMessage};
use serde::Deserialize;
use surf::http::Url;
use once_cell::sync::OnceCell;
use crate::warn;

#[derive(Deserialize, Debug)]
pub(crate) struct TwilioConfig {
    pub(crate) enable_call: bool,
    pub(crate) enable_text: bool,
    twilio_id: Option<String>,
    twilio_auth: Option<String>,
    to_phone: Option<String>,
    from_phone: Option<String>,
    default_callback: Option<Url>,
}

pub(crate) static TW_CONFIG: OnceCell<TwilioConfig> = OnceCell::new();

// must set config prior to using
pub(crate) fn set_twilio_config(config: TwilioConfig) {
    TW_CONFIG.set(config).expect("set twilio config"); 
}

/// Time to call home
pub(crate) async fn make_the_call(callback: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tw_client = get_client();
    let config = TW_CONFIG.get().expect("get TW_CONFIG");
    let from_phone = match &config.from_phone {
        Some(phone) => phone,
        None => {
            warn("twilio", "No 'from phone' number provided.");
            return Ok(());
        },
    };
    let to_phone = match &config.to_phone {
        Some(phone) => phone,
        None => {
            warn("twilio", "No 'to phone' number provided.");
            return Ok(());
        },
    };

    let call = tw_client
        .make_call(OutboundCall::new(
            &from_phone,
            &to_phone,
            callback,
        ));

    call.await?;
    Ok(())
}

/// Text message
pub(crate) async fn send_text(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tw_client = get_client();
    let config = TW_CONFIG.get().expect("get TW_CONFIG");
    let from_phone = match &config.from_phone {
        Some(phone) => phone,
        None => {
            warn("twilio", "No 'from phone' number provided.");
            return Ok(());
        },
    };
    let to_phone = match &config.to_phone {
        Some(phone) => phone,
        None => {
            warn("twilio", "No 'to phone' number provided.");
            return Ok(());
        },
    };

    let msg = tw_client
        .send_message(OutboundMessage::new(
            &from_phone,
            &to_phone,
            message,
        ));

    msg.await?;
    Ok(())
}

fn get_client() -> Lazy<twilio::Client> {
    // create static client
    Lazy::new(|| {
        let config = TW_CONFIG.get().expect("get TW_CONFIG");
        let id = match &config.twilio_id {
            Some(n) => n,
            None => {
                warn("twilio", "No twilio id/auth provided");
                ""
            },
        };
        let auth = match &config.twilio_auth {
            Some(n) => n,
            None => {
                warn("twilio", "No twilio id/auth provided");
                ""
            },
        };

        twilio::Client::new(id, auth)
    })
}
