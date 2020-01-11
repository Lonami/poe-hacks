use libflate::gzip;

use native_tls::{TlsConnector, TlsStream};
use std::io::{Read, Write};
use std::net::TcpStream;

// Possible optimizations:
// * pre-calculate serialized data length and use with capacity
// * avoid strings and just use a byte buffer
// * avoid format!() and just extend() sections

// TODO Don't hardcode league, fetch from http://api.pathofexile.com/leagues

fn quote_plus(string: &str) -> String {
    let mut buffer = [0; 4];
    let mut result = String::with_capacity(string.len());
    string.chars().for_each(|c| match c {
        ' ' => result.push('+'),
        '-' | '.' | '_' | '~' => result.push(c),
        '0'..='9' => result.push(c),
        'A'..='Z' => result.push(c),
        'a'..='z' => result.push(c),
        _ => {
            c.encode_utf8(&mut buffer).as_bytes().iter().for_each(|b| {
                result.push_str(&format!("%{:x}", b));
            });
        }
    });
    result
}

fn url_encode(data: &[(&str, &str)]) -> String {
    let mut result = String::new();
    if data.len() > 0 {
        let (key, value) = data[0];
        result.push_str(&format!("{}={}", key, quote_plus(value)));

        for (key, value) in &data[1..] {
            result.push_str(&format!("&{}={}", key, quote_plus(value)));
        }
    }
    result
}

fn connect_to_poe_trade() -> Result<TlsStream<TcpStream>, String> {
    let connector = TlsConnector::new().map_err(|e| format!("{:?}", e))?;
    let stream = TcpStream::connect("poe.trade:443").map_err(|e| format!("{:?}", e))?;
    let stream = connector
        .connect("poe.trade", stream)
        .map_err(|e| format!("{:?}", e))?;

    Ok(stream)
}

fn post<S: Read + Write>(stream: &mut S, data: &[(&str, &str)]) -> Result<String, String> {
    let payload = url_encode(data);

    let request = format!("POST /search HTTP/1.1\r
Host: poe.trade\r
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0\r
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r
Accept-Language: en\r
Accept-Encoding: gzip, deflate, br\r
Content-Type: application/x-www-form-urlencoded\r
Content-Length: {len}\r
Origin: https://poe.trade\r
Connection: keep-alive\r
Referer: https://poe.trade/\r
Cookie: __vrz=1.16.10; league=Hardcore%20Metamorph; live_notify_sound=0; live_notify_browser=0; live_frequency=0\r
Upgrade-Insecure-Requests: 1\r
\r
{payload}", len = payload.len(), payload = payload);

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("{:?}", e))?;

    let mut response = vec![0u8; 4096];
    let read = stream.read(&mut response).map_err(|e| format!("{:?}", e))?;
    Ok(String::from_utf8_lossy(&response[..read]).to_string())
}

fn find_redirect(string: &str) -> Option<String> {
    if let Some(start) = string.find("Location:") {
        let start = start + 10;
        if let Some(end) = string[start..].find("\r\n") {
            let end = start + end;
            return Some(string[start..end].into());
        }
    }
    None
}

fn get<S: Read + Write>(stream: &mut S, path: &str) -> Result<String, String> {
    let request = format!("GET {path} HTTP/1.1\r
Host: poe.trade\r
User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:72.0) Gecko/20100101 Firefox/72.0\r
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r
Accept-Language: en\r
Accept-Encoding: gzip, deflate, br\r
Referer: https://poe.trade/\r
Connection: close\r
Cookie: __vrz=1.16.10; league=Hardcore%20Metamorph; live_notify_sound=0; live_notify_browser=0; live_frequency=0\r
Upgrade-Insecure-Requests: 1\r
\r
", path = path);

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("{:?}", e))?;

    let mut response = vec![];
    stream
        .read_to_end(&mut response)
        .map_err(|e| format!("{:?}", e))?;

    let mut len_start = match (0..response.len() - 4).find(|&i| &response[i..i + 4] == b"\r\n\r\n")
    {
        Some(x) => x + 4, // skip b"\r\n\r\n"
        None => return Err("could not find empty newline in response".into()),
    };

    let mut gzipped: Vec<u8> = Vec::with_capacity(response.len() - len_start);
    loop {
        // Find the hexadecimal length in the current line
        let len_end =
            match (len_start..response.len() - 2).find(|&i| &response[i..i + 2] == b"\r\n") {
                Some(x) => x,
                None => return Err("could not find end of chunked length in response".into()),
            };

        // Parse the length and break if it's 0
        let len_str = String::from_utf8_lossy(&response[len_start..len_end]);
        let chunk_len = usize::from_str_radix(&len_str, 16)
            .map_err(|e| format!("{:?} is not a valid hex number: {:?}", len_str, e))?;

        if chunk_len == 0 {
            break;
        }

        // Save the data in its own buffer
        let data_start = len_end + 2; // skip b"\r\n"
        let data_end = data_start + chunk_len;
        gzipped.extend(&response[data_start..data_end]);

        // Prepare the start offset for the next iteration
        len_start = data_end + 2; // skip b"\r\n"
        if len_start >= response.len() {
            return Err("reached eof without finishing the data".into());
        }
    }

    let mut decoder = gzip::Decoder::new(&gzipped[..]).map_err(|e| format!("{:?}", e))?;
    let mut result = Vec::new();
    decoder
        .read_to_end(&mut result)
        .map_err(|e| format!("{:?}", e))?;

    Ok(String::from_utf8_lossy(&result).to_string())
}

fn query_poe_trade(data: &[(&str, &str)]) -> Result<String, String> {
    let mut stream = connect_to_poe_trade()?;
    let response = post(&mut stream, data)?;
    let redirect = match find_redirect(&response) {
        Some(x) => x,
        None => {
            return Err(format!(
                "no redirect location found in response:\n{}",
                response
            ))
        }
    };
    let path = match redirect.find("poe.trade") {
        Some(pos) => &redirect[pos + 9..],
        None => return Err(format!("bad redirect lacks hostname: {}", redirect)),
    };

    get(&mut stream, path)
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
