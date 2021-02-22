use crate::caller::*;
use crate::{info, warn, LOG};
use async_std::task;
use chrono::Local;
use futures::join;
use once_cell::unsync::{Lazy, OnceCell};
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;
use surf::http::{Request, Url};

const SURF_CLIENT: Lazy<surf::Client> = Lazy::new(|| surf::Client::new());

// everything owned since we load from config file
#[derive(Deserialize, Debug)]
pub(crate) struct Lookout {
    pub(crate) name: String,
    url: Url,
    regex: String,
    expected_matches: usize,
    timeout: u64,
    selectors: Option<Vec<String>>,
    headers: Option<HashMap<String, String>>,
}

pub(crate) fn search(text: &str, re: &str) -> usize {
    let mut map: Lazy<HashMap<&str, OnceCell<Regex>>> = Lazy::new(HashMap::new);
    let re_cell = map.entry(re).or_insert(OnceCell::new());
    let re = re_cell.get_or_init(|| Regex::new(re).expect("Failed to parse regex"));
    re.captures_iter(text).count()
}

pub(crate) async fn check_and_alert(
    name: &str,
    num_matches: usize,
    expected: usize,
    alert_url: &str,
    alert_delay: u64,
) -> Result<bool, Box<dyn std::error::Error>> {
    if num_matches != expected {
        if TW_CONFIG.get().expect("TW_CONFIG get").enable_call {
            make_the_call(alert_url).await?;
        }
        if TW_CONFIG.get().expect("TW_CONFIG get").enable_text {
            send_text(alert_url).await?;
        }
        warn(
            name,
            &format!("expected {} found {}", expected, num_matches),
        );
        warn(name, alert_url);
        task::sleep(Duration::from_secs(alert_delay)).await;
        Ok(true)
    } else {
        Ok(false)
    }
}

impl<'a> Lookout {
    pub(crate) async fn scrape_timeout(
        &self,
        alert_delay: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let scraper = self.scrape(alert_delay);
        // delay instead of timer so we seem less like a bot??
        let sleeper = task::sleep(Duration::from_secs(self.timeout));
        let (scraper, _) = join!(scraper, sleeper);
        scraper
    }

    pub(crate) async fn scrape(&self, alert_delay: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut haystack = match &self.headers {
            None => surf::get(&self.url).await?.body_string().await?,
            Some(headers) => {
                let mut req = Request::get(self.url.clone()); // TODO
                for (k, v) in headers {
                    let k: &str = &k;
                    let v: &str = &v;
                    req.insert_header(k, v);
                }
                let mut resp = SURF_CLIENT.send(req).await?;
                resp.body_string().await?
            }
        };
        if let Some(selectors) = &self.selectors {
            let html = Html::parse_document(&haystack);
            // TODO currently only taking the first selector hit
            haystack = selectors
                .iter()
                .map(|s| {
                    html.select(&Selector::parse(s).unwrap())
                        .next()
                        .expect("couldnt find selector")
                        .html()
                })
                .collect::<Vec<_>>()
                .join("\n");
        }
        let matches = search(&haystack, &self.regex);
        info(
            &self.name,
            &format!("Found {} instances of '{}'", matches, self.regex),
        );
        if *LOG.get().expect("get LOG") {
            let time = Local::now();
            // log body
            let mut file = OpenOptions::new().create(true).append(true).open(format!(
                "{}-{}.html",
                self.name,
                time.timestamp()
            ))?;
            file.write(haystack.as_bytes())?;
        }
        let _ = check_and_alert(
            &self.name,
            matches,
            self.expected_matches,
            self.url.to_string().as_ref(),
            alert_delay,
        )
        .await?;
        Ok(())
    }
}
