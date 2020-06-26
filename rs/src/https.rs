use json_minimal::Json;
use std::sync::Mutex;

lazy_static! {
    static ref LEAGUE: Mutex<Option<String>> = Mutex::new(None);
}

fn query_poe_trade(data: &[(&str, &str)]) -> Result<String, String> {
    let response = attohttpc::post("https://poe.trade/search")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0",
        )
        .params(data)
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
        .bytes()
        .map_err(|e| format!("{:?}", e))?;

    let json = Json::parse(&response).map_err(|e| e.1.to_string())?;

    // json[4]["id"]
    match json {
        Json::ARRAY(array) => {
            match array
                .get(5)
                .ok_or(format!("{} are not enough leagues", array.len()))?
                .get("id")
                .ok_or(format!("no league id found"))?
            {
                Json::OBJECT { value, .. } => match value.unbox() {
                    Json::STRING(string) => return Ok(string.clone()),
                    _ => Err(format!("league id was not a string")),
                },
                _ => Err(format!("get failed to get an object")),
            }
        }
        _ => Err(format!("api did not return an array")),
    }
}

fn parse_currency(string: &str) -> Option<f64> {
    let mut it = string.split_whitespace();
    if let Some(amount) = it.next() {
        if let Ok(amount) = amount.parse() {
            if let Some(kind) = it.next() {
                return Some(match kind {
                    "exalted" => amount * 64.0,
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
