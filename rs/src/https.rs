use std::collections::HashMap;
use std::sync::Mutex;
use tinyjson::JsonValue as Json;

lazy_static! {
    static ref LEAGUE: Mutex<Option<String>> = Mutex::new(None);
    static ref EXALT_PRICE: Mutex<Option<f64>> = Mutex::new(None);
}

// TODO how to let the user change this? "sc" flag in .key file?
const HARDCORE: bool = true;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0";

fn query_poe_trade(data: &[(&str, &str)]) -> Result<String, String> {
    let response = attohttpc::post("https://poe.trade/search")
        .header("User-Agent", USER_AGENT)
        .params(data)
        .send()
        .map_err(|e| format!("{:?}", e))?
        .bytes()
        .map_err(|e| format!("{:?}", e))?;

    Ok(String::from_utf8_lossy(&response).to_string())
}

fn query_exalt_prices(league: &str) -> Result<String, String> {
    let response = attohttpc::get("https://currency.poe.trade/search")
        .header("User-Agent", USER_AGENT)
        .params(&[
            ("league", league),
            ("online", "x"),
            ("want", "4"),
            ("have", "6"),
        ])
        .send()
        .map_err(|e| format!("{:?}", e))?
        .bytes()
        .map_err(|e| format!("{:?}", e))?;

    Ok(String::from_utf8_lossy(&response).to_string())
}

fn current_league() -> Result<String, String> {
    let response = attohttpc::get("http://api.pathofexile.com/leagues")
        .send()
        .map_err(|e| format!("{:?}", e))?
        .text()
        .map_err(|e| format!("{:?}", e))?;

    let json = response.parse::<Json>().map_err(|e| e.to_string())?;

    // json[#]["id"]
    json.get::<Vec<_>>()
        .ok_or("api did not return an array".to_string())?
        .get(if HARDCORE { 5 } else { 4 })
        .ok_or("not enough items in array".to_string())?
        .get::<HashMap<_, _>>()
        .ok_or("failed to get an object from array".to_string())?
        .get("id")
        .ok_or("no league id found in object")?
        .get::<String>()
        .ok_or("league id was not a string".to_string())
        .map(|id| id.to_string())
}

fn find_attrs<'a>(html: &'a str, attr: &str) -> Vec<&'a str> {
    html.match_indices(attr)
        .flat_map(|(pos, _)| {
            let pos = pos + attr.len();
            match html[pos..].find('"') {
                Some(quote_end) => Some(&html[pos..pos + quote_end]),
                None => None,
            }
        })
        .collect()
}

fn find_exalt_prices(league: &str) -> Result<Vec<f64>, String> {
    let html = query_exalt_prices(league)?;
    let sell_prices = find_attrs(&html, "data-sellvalue=\"");
    let buy_prices = find_attrs(&html, "data-buyvalue=\"");

    Ok(sell_prices
        .into_iter()
        .zip(buy_prices.into_iter())
        .flat_map(
            |(sell, buy)| match (sell.parse::<f64>(), buy.parse::<f64>()) {
                (Ok(sell), Ok(buy)) => Some(sell / buy),
                _ => None,
            },
        )
        .collect())
}

fn parse_currency(string: &str) -> Option<f64> {
    let mut it = string.split_whitespace();
    if let Some(amount) = it.next() {
        if let Ok(amount) = amount.parse() {
            if let Some(kind) = it.next() {
                return Some(match kind {
                    "exalted" => amount * EXALT_PRICE.lock().unwrap().unwrap_or(64.0),
                    "chaos" => amount,
                    "fusing" => amount / 2.0,
                    "alchemy" => amount / 4.0,
                    "alteration" => amount / 8.0,
                    "chromatic" => amount / 8.0,
                    "chisel" => amount / 8.0,
                    "chance" => amount / 8.0,
                    "blessed" => amount / 16.0,
                    _ => return None,
                });
            }
        }
    }
    None
}

pub fn find_prices(data: &[(&str, &str)]) -> Result<Vec<f64>, String> {
    let html = query_poe_trade(&data)?;
    let attr = "data-buyout=\"";

    Ok(html
        .match_indices(attr)
        .flat_map(|(pos, _)| {
            let pos = pos + attr.len();
            match html[pos..].find('"') {
                Some(quote_end) => parse_currency(&html[pos..pos + quote_end]),
                None => None,
            }
        })
        .collect())
}

pub fn find_unique_prices(unique: &str) -> Result<Vec<f64>, String> {
    let mut league = LEAGUE.lock().unwrap();
    if league.is_none() {
        league.replace(current_league()?);
    }

    let league = league.as_ref().unwrap();

    {
        // update the exalt price if it's missing (else we'll use a default)
        let mut exalt_price = EXALT_PRICE.lock().unwrap();
        if exalt_price.is_none() {
            match find_exalt_prices(league) {
                Ok(prices) => {
                    exalt_price.replace(prices.iter().take(5).sum::<f64>() / 5.0);
                }
                Err(e) => {
                    eprintln!("failed to get exalt prices: {:?}", e);
                }
            }
        }
    }

    find_prices(&[
        ("league", league),
        ("type", ""),
        ("base", ""),
        ("name", unique),
        ("dmg_min", ""),
        ("dmg_max", ""),
        ("aps_min", ""),
        ("aps_max", ""),
        ("crit_min", ""),
        ("crit_max", ""),
        ("dps_min", ""),
        ("dps_max", ""),
        ("edps_min", ""),
        ("edps_max", ""),
        ("pdps_min", ""),
        ("pdps_max", ""),
        ("armour_min", ""),
        ("armour_max", ""),
        ("evasion_min", ""),
        ("evasion_max", ""),
        ("shield_min", ""),
        ("shield_max", ""),
        ("block_min", ""),
        ("block_max", ""),
        ("sockets_min", ""),
        ("sockets_max", ""),
        ("link_min", ""),
        ("link_max", "4"), // we just want the base unique price so find non-linked
        ("sockets_r", ""),
        ("sockets_g", ""),
        ("sockets_b", ""),
        ("sockets_w", ""),
        ("linked_r", ""),
        ("linked_g", ""),
        ("linked_b", ""),
        ("linked_w", ""),
        ("rlevel_min", ""),
        ("rlevel_max", ""),
        ("rstr_min", ""),
        ("rstr_max", ""),
        ("rdex_min", ""),
        ("rdex_max", ""),
        ("rint_min", ""),
        ("rint_max", ""),
        ("mod_name", ""),
        ("mod_min", ""),
        ("mod_max", ""),
        ("mod_weight", ""),
        ("group_type", "And"),
        ("group_min", ""),
        ("group_max", ""),
        ("group_count", "1"),
        ("q_min", ""),
        ("q_max", ""),
        ("level_min", ""),
        ("level_max", ""),
        ("ilvl_min", ""),
        ("ilvl_max", ""),
        ("rarity", ""),
        ("progress_min", ""),
        ("progress_max", ""),
        ("sockets_a_min", ""),
        ("sockets_a_max", ""),
        ("map_series", ""),
        ("altart", ""),
        ("identified", ""),
        ("corrupted", ""),
        ("crafted", ""),
        ("enchanted", ""),
        ("fractured", ""),
        ("synthesised", ""),
        ("mirrored", ""),
        ("veiled", ""),
        ("shaper", ""),
        ("elder", ""),
        ("crusader", ""),
        ("redeemer", ""),
        ("hunter", ""),
        ("warlord", ""),
        ("seller", ""),
        ("thread", ""),
        ("online", "x"),
        ("capquality", "x"),
        ("buyout_min", ""),
        ("buyout_max", ""),
        ("buyout_currency", ""),
        ("has_buyout", ""),
        ("exact_currency", ""),
    ])
}
