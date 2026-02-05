#![allow(dead_code)]
pub mod languages;
pub mod result;

use std::sync::{Mutex, LazyLock};
use serde::Serialize;

use self::{languages::Language, result::GoogleTranslateResult};

pub static SESSION: LazyLock<Mutex<GoogleTranslateSession>> = LazyLock::new(|| Mutex::new(GoogleTranslateSession::default()));
static LENGTH_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^(\d+)").unwrap());

#[derive(Debug, Clone, Default, Serialize)]
pub struct GoogleTranslateSession {
    rpcids: String,
    source_path: String,
    f_sid: String,
    bl: String,
    hl: String,
    soc_app: u8,
    soc_platform: u8,
    soc_device: u8,
    reqid: u32,
    rt: String,
}

fn extract(key: &str, res: &str) -> String {
    let pattern = format!(r#""{}":"(.*?)""#, regex::escape(key));
    let re = regex::Regex::new(&pattern).unwrap();
    re.captures(res)
        .and_then(|caps| caps.get(1))
        .map_or(String::new(), |m| m.as_str().to_owned())
}

async fn get_session() -> Result<GoogleTranslateSession, Box<dyn std::error::Error>> {
    let initial_response = reqwest::get("https://translate.google.com/")
        .await?;

    if initial_response.status() != reqwest::StatusCode::OK {
        return Err("Failed to connect to Google Translate".into());
    }

    let init_response_text = initial_response.text()
        .await?;

    Ok(GoogleTranslateSession {
        rpcids: "MkEWBc".into(),
        source_path: "/".into(),
        f_sid: extract("FdrFJe", &init_response_text),
        bl: extract("cfb2h", &init_response_text),
        hl: "en".into(),
        soc_app: 1,
        soc_platform: 1,
        soc_device: 1,
        reqid: rand::random(),
        rt: "c".into(),
    })
}

pub async fn translate(
    text: &str,
    source_lang: Language,
    target_lang: Language,
    autocorrect: bool
) -> Result<GoogleTranslateResult, Box<dyn std::error::Error>> {
    let session = SESSION.try_lock()
        .map_err(|_| "Failed to acquire lock for Google Translate session")?
        .clone();

    // Prepare the RPC request payload (f.req)
    let inner_list = vec![
        vec![
            serde_json::Value::String(text.to_owned()),
            serde_json::Value::String(source_lang.code.clone()),
            serde_json::Value::String(target_lang.code.clone()),
            serde_json::Value::Bool(autocorrect)
        ],
        vec![serde_json::Value::Null]
    ];

    let payload = [[[
        serde_json::Value::String(session.rpcids.clone()),
        serde_json::Value::String(serde_json::to_string(&inner_list)?),
        serde_json::Value::Null,
        serde_json::Value::String("generic".to_owned())
    ]]];

    let session_query = [
        ("rpcids", session.rpcids.as_str()),
        ("source-path", session.source_path.as_str()),
        ("f.sid", session.f_sid.as_str()),
        ("bl", session.bl.as_str()),
        ("hl", session.hl.as_str()),
        ("soc-app", &session.soc_app.to_string()),
        ("soc-platform", &session.soc_platform.to_string()),
        ("soc-device", &session.soc_device.to_string()),
        ("rt", session.rt.as_str()),
        ("_reqid", &session.reqid.to_string()),
    ];

    let translation_response = reqwest::Client::new()
        .post("https://translate.google.com/_/TranslateWebserverUi/data/batchexecute")
        .query(&session_query)
        .form(&[("f.req", serde_json::to_string(&payload)?)])
        .send()
        .await?;
    
    // Parse the response text into a sane structure
    // !! ABSOLUTE MAGIC NUMBER HELL, THIS COULD BREAK AT ANY TIME !!
    let mut result = GoogleTranslateResult::default();
    let raw_response = translation_response.text().await?;
    let raw_response = &raw_response[6..]; // Skip the ")]}'" garbage

    let captures = LENGTH_REGEX.captures(raw_response);
    if let Some(caps) = captures {
        // Removing everything after the second linebreak appears to be more reliable than
        // blindly trusting the length we're given by the RPC response.
        let start = caps.get(0).unwrap().end() + 1;
        let mut chunk = &raw_response[start..];
        if let Some(newline_pos) = chunk.find('\n') {
            chunk = &chunk[..newline_pos];
        } else {
            return Err("No newline found! Something probably changed internally.".into());
        }

        let data: Vec<serde_json::Value> = serde_json::from_str(chunk)?;
        let json: Vec<serde_json::Value> = serde_json::from_str(data.first().and_then(|v| v.get(2)).unwrap().as_str().unwrap())?;

        if let Some(pronunciation) = json.get(1).and_then(|v| v.get(0)).and_then(|v| v.get(1)) {
            result.pronunciation = pronunciation.as_str().unwrap_or("").to_owned();
        }

        // Detect source language
        if json[0][1].is_array() {
            result.from.language_did_you_mean = true;
            result.from.language.code = json[0][1][1][0].as_str().unwrap_or("").to_owned();
        } else if json[1][3] == "auto" {
            result.from.language.code = json[2].as_str().unwrap_or("").to_owned();
        } else {
            result.from.language.code = json[1][3].as_str().unwrap_or("").to_owned();
        }
        result.from.language.name = languages::get_language_name(&result.from.language.code)
            .unwrap_or_else(|| "Unknown".to_owned());

        // Build target text and language code
        if json[1][0][0][5].is_array() {
            result.to.text = json[1][0][0][5]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.get(0)?.as_str())
                .collect::<Vec<&str>>()
                .join(" ");
        } else {
            result.to.text = json[1][0][0][0].as_str().unwrap_or("").to_owned();
        }
        result.to.language.code = json[1][1].as_str().unwrap_or("").to_owned();
        result.to.language.name = languages::get_language_name(&result.to.language.code)
            .unwrap_or_else(|| "Unknown".to_owned());

        // Autocorrect / didYouMean for source text, if any
        if json[0][1].is_array() && !json[0][1].as_array().unwrap().is_empty() {
            let obj = &json[0][1][0];

            if obj.is_array() && !obj.as_array().unwrap().is_empty() && obj[0].is_array() && obj[0].as_array().unwrap().len() > 1 {
                let cleaned = obj[0][1].as_str().unwrap_or("")
                    .replace("<b>", "[")
                    .replace("</b>", "]");
            
                result.from.text = cleaned;
            }

            if obj[2].is_i64() && obj[2].as_i64().unwrap() == 1 {
                result.from.text_auto_corrected = true;
            } else {
                result.from.text_did_you_mean = true;
            }
        }
    }

    Ok(result)
}

pub async fn refresh_session() -> Result<(), Box<dyn std::error::Error>> {
    let new_session = get_session().await?;
    let mut session_lock = SESSION.lock().unwrap();
    *session_lock = new_session;
    Ok(())
}

pub fn activate() {
    tokio::spawn(async {
        let refresh_session_result = refresh_session().await;
        if let Err(error) = refresh_session_result {
            error!(%error, "Failed to initialize Google Translate session");
        } else {
            info!("Google Translate session initialized");
        }
    });
}