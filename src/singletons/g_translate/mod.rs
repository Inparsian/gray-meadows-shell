#![allow(dead_code)]
pub mod language;
pub mod result;

use std::sync::Mutex;
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::singletons::g_translate::{language::Language, result::GoogleTranslateResult};

pub static SESSION: Lazy<Mutex<GoogleTranslateSession>> = Lazy::new(|| Mutex::new(GoogleTranslateSession::default()));
static LENGTH_REGEX: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"^\d+").unwrap());

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
        .map_or(String::new(), |m| m.as_str().to_string())
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
            serde_json::Value::String(text.to_string()),
            serde_json::Value::String(source_lang.code.to_string()),
            serde_json::Value::String(target_lang.code.to_string()),
            serde_json::Value::Bool(autocorrect)
        ],
        vec![serde_json::Value::Null]
    ];

    let payload = [[[
        serde_json::Value::String(session.rpcids.clone()),
        serde_json::Value::String(serde_json::to_string(&inner_list)?),
        serde_json::Value::Null,
        serde_json::Value::String("generic".to_string())
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
    let raw_response = translation_response.text()
        .await?;
    let raw_response = &raw_response[6..]; // Skip the ")]}'"

    let captures = LENGTH_REGEX.captures(raw_response);
    if let Some(caps) = captures {
        let length = caps[1].parse().unwrap_or(0);
        let start = caps.get(0).unwrap().end();
        let end = start + length;
        let chunk = &raw_response[start..end];
        let data: Vec<serde_json::Value> = serde_json::from_str(chunk)?;
        let j: Vec<serde_json::Value> = serde_json::from_str(data.first().and_then(|v| v.get(2)).unwrap().as_str().unwrap())?;

        if let Some(pronunciation) = j.get(1).and_then(|v| v.get(0)).and_then(|v| v.get(1)) {
            result.pronunciation = pronunciation.as_str().unwrap_or("").to_string();
        }

        // Detect source language
        if j[0][1].is_array() {
            result.from.language_did_you_mean = true;
            result.from.language.code = j[0][1][1][0].as_str().unwrap_or("").to_string();
        } else if j[1][3] == "auto" {
            result.from.language.code = j[2].as_str().unwrap_or("").to_string();
        } else {
            result.from.language.code = j[1][3].as_str().unwrap_or("").to_string();
        }
        result.from.language.name = language::get_language_name(&result.from.language.code)
            .unwrap_or_else(|| "Unknown".to_string());

        // Build target text and language code
        if j[1][0][0][5].is_array() {
            result.to.text = j[1][0][0][5]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.get(0).and_then(|s| s.as_str()))
                .collect::<Vec<&str>>()
                .join(" ");
        } else {
            result.to.text = j[1][0][0][0].as_str().unwrap_or("").to_string();
        }
        result.to.language.code = j[1][1].as_str().unwrap_or("").to_string();
        result.to.language.name = language::get_language_name(&result.to.language.code)
            .unwrap_or_else(|| "Unknown".to_string());

        // Autocorrect / didYouMean for source text, if any
        if j[0][1].is_array() && !j[0][1].as_array().unwrap().is_empty() {
            let obj = &j[0][1][0];

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

pub fn activate() {
    tokio::spawn(async {
        let session = get_session().await;
        
        if let Ok(session) = session {
            let mut session_lock = SESSION.lock().unwrap();
            *session_lock = session;

            println!("Google Translate session initialized successfully.");
        } else {
            eprintln!("Failed to initialize Google Translate session: {:?}", session);
        }
    });
}