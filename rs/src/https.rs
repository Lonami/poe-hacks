// Possible optimizations:
// * pre-calculate serialized data length and use with capacity
// * avoid strings and just use a byte buffer
// * avoid format!() and just extend() sections

// TODO Don't hardcode league, fetch from http://api.pathofexile.com/leagues

fn query_poe_trade(data: &[(&str, &str)]) -> Result<String, String> {
    let response = attohttpc::post("https://poe.trade/search")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0")
        .header("Cookie", " __vrz=1.16.10; league=Hardcore%20Metamorph; live_notify_sound=0; live_notify_browser=0; live_frequency=0")
        .params(data)
        .send()
        .map_err(|e| format!("{:?}", e))?
        .bytes()
        .map_err(|e| format!("{:?}", e))?;

    Ok(String::from_utf8_lossy(&response).to_string())
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
    find_prices(&[
        ("league", "Hardcore Metamorph"),
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
