Lookout
=======

An asynchronous scraper utility. Given a TOML file describing sites, selectors, regex, and expected
regex matches, this will continuously scrape and call your phone using Twilio when expected matches
are not met (if configured).

For example, this could scrape for product stock and call you when the expected 'out of stock'
elements on the page are no longer found.

DISCLAIMER: Always abide by limits/regulations when scraping. The example below is for illustrative
purposes only.

### Example
```toml
log = false

# in seconds
error_delay = 300 # not yet implemented
alert_delay = 300 # not yet implemented

allowed_errors = 20 # not yet implemented

[twilio]
enable_call = false
enable_text = false

[[lookout]]
name = "newegg"
url = "https://www.newegg.com/product
regex = "OUT OF STOCK|SOLD OUT|Sold Out"
expected_matches = 3
selectors = [ "#app > div.page-content > div.page-section > div > div > div.row-side > div.product-buy-box",
              "#app > div.page-content > div.page-section > div > div > div.row-body > div.product-main.display-flex > div
timeout = 180

[[lookout]]
name = "b&h"
url = "https://www.bhphotovideo.com/some/cool/product
regex = "Notify When Available"
expected_matches = 1
timeout = 180

[[lookout]]
name = "bestbuy"
url = "https://www.bestbuy.com/site/something/
regex = "Sold Out"
expected_matches = 2
timeout = 180

[lookout.headers]
"User-Agent" = "Wget/1.20.1 (linux-gnu)"
"Accept-Encoding" = "identity"
"ACCEPT" = "*/*"
"Connection" = "Keep-Alive"
```
