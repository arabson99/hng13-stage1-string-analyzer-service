use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use chrono::Utc;
use lazy_static::lazy_static;

lazy_static! {
    static ref STORE: Mutex<HashMap<String, AnalyzedString>> = Mutex::new(HashMap::new());
}

#[derive(Serialize, Clone)]
struct AnalyzedString {
    id: String,
    value: String,
    properties: Properties,
    created_at: String,
}

#[derive(Serialize, Clone)]
struct Properties {
    length: usize,
    is_palindrome: bool,
    unique_characters: usize,
    word_count: usize,
    sha256_hash: String,
    character_frequency_map: HashMap<char, usize>,
}

#[derive(Deserialize)]
struct InputString {
    value: String,
}

fn analyze_string(value: &str) -> Properties {
    let length = value.chars().count();
    // Case-insensitive palindrome check (ignoring case)
    let lowercase: String = value.to_lowercase();
    let rev: String = lowercase.chars().rev().collect();
    let is_palindrome = lowercase == rev;

    let unique_characters: HashSet<char> = value.chars().collect();
    let word_count = value.split_whitespace().count();

    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let sha256_hash = hex::encode(hasher.finalize());

    let mut freq = HashMap::new();
    for ch in value.chars() {
        *freq.entry(ch).or_insert(0) += 1;
    }

    Properties {
        length,
        is_palindrome,
        unique_characters: unique_characters.len(),
        word_count,
        sha256_hash: sha256_hash.clone(),
        character_frequency_map: freq,
    }
}

// POST /strings
async fn create_string(body: web::Bytes) -> impl Responder {
    let parsed: Result<InputString, _> = serde_json::from_slice(&body);
    match parsed {
        Ok(body) => {
            let value = body.value.trim();
            if value.is_empty() {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Missing or empty 'value' field"
                }));
            }

            let props = analyze_string(value);
            let id = props.sha256_hash.clone();

            let mut store = STORE.lock().unwrap();
            if store.contains_key(&id) {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": "String already exists"
                }));
            }

            let entry = AnalyzedString {
                id: id.clone(),
                value: value.to_string(),
                properties: props.clone(),
                created_at: Utc::now()
                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            };
            store.insert(id.clone(), entry.clone());

            HttpResponse::Created().json(entry)
        }
        Err(_) => HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": r#"Invalid data type for "value" (must be string)"#
        })),
    }
}


// GET /strings/{string_value}
async fn get_string(path: web::Path<String>) -> impl Responder {
    let val = path.into_inner();
    // Compute hash
    let mut hasher = Sha256::new();
    hasher.update(val.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let store = STORE.lock().unwrap();
    if let Some(entry) = store.get(&hash) {
        HttpResponse::Ok().json(entry.clone())
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "String does not exist in the system"
        }))
    }
}

// GET /strings (with optional filters and validation)
async fn get_all(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let store = STORE.lock().unwrap();
    let mut all: Vec<AnalyzedString> = store.values().cloned().collect();

    // Validate and apply filters
    if let Some(is_pal) = query.get("is_palindrome") {
        match is_pal.as_str() {
            "true" | "false" => {
                let b = is_pal == "true";
                all.retain(|e| e.properties.is_palindrome == b);
            }
            _ => {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid query parameter values or types"
                }));
            }
        }
    }

    for key in ["min_length", "max_length", "word_count"] {
        if let Some(val) = query.get(key) {
            if let Ok(num) = val.parse::<usize>() {
                match key {
                    "min_length" => all.retain(|e| e.properties.length >= num),
                    "max_length" => all.retain(|e| e.properties.length <= num),
                    "word_count" => all.retain(|e| e.properties.word_count == num),
                    _ => {}
                }
            } else {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid query parameter values or types"
                }));
            }
        }
    }

    if let Some(cont) = query.get("contains_character") {
        if cont.chars().count() == 1 {
            let ch = cont.chars().next().unwrap();
            all.retain(|e| e.properties.character_frequency_map.contains_key(&ch));
        } else {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid query parameter values or types"
            }));
        }
    }

    HttpResponse::Ok().json(serde_json::json!({
        "data": all,
        "count": all.len(),
        "filters_applied": query.into_inner()
    }))
}


// DELETE /strings/{string_value}
async fn delete_string(path: web::Path<String>) -> impl Responder {
    let val = path.into_inner();
    let mut hasher = Sha256::new();
    hasher.update(val.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let mut store = STORE.lock().unwrap();
    if store.remove(&hash).is_some() {
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "String does not exist in the system"
        }))
    }
}

// Natural language filter 
async fn filter_nl(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let q_raw = query.get("query");
    if q_raw.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Missing 'query' parameter"
        }));
    }

    let q = q_raw.unwrap().trim().to_lowercase();

    // handle empty or invalid queries
    if q.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Unable to parse natural language query"
        }));
    }

    // parsed filters
    let mut want_palindrome: Option<bool> = None;
    let mut want_word_count: Option<usize> = None;
    let mut min_length: Option<usize> = None;
    let mut contains_character: Option<char> = None;

    // Palindromic or Non-palindromic
    if q.contains("non-palindromic") {
        want_palindrome = Some(false);
    } else if q.contains("palindromic") || q.contains("palindrome") {
        want_palindrome = Some(true);
    }

    // Single word
    if q.contains("single word") {
        want_word_count = Some(1);
    }

    //  longer than N characters
    if let Some(pos) = q.find("longer than") {
        let after = &q[pos + "longer than".len()..];
        if let Some(num_str) = after.split_whitespace().next() {
            if let Ok(n) = num_str.parse::<usize>() {
                min_length = Some(n + 1); // > N means min_length = N+1
            }
        }
    }

    // containing the letter X
    if let Some(pos) = q.find("containing the letter") {
        let after = &q[pos + "containing the letter".len()..];
        if let Some(letter) = after.trim().chars().next() {
            if letter.is_alphabetic() {
                contains_character = Some(letter);
            }
        }
    }

    // Contain the first vowel
    if q.contains("contain the first vowel") {
        contains_character = Some('a'); // heuristic
    }

    // detect conflicting filters
    if q.contains("non-palindromic") && q.contains("palindromic") {
        return HttpResponse::UnprocessableEntity().json(serde_json::json!({
            "error": "Query parsed but resulted in conflicting filters"
        }));
    }

    // if no recognized filters found
    if want_palindrome.is_none()
        && want_word_count.is_none()
        && min_length.is_none()
        && contains_character.is_none()
    {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Unable to parse natural language query"
        }));
    }

    // apply filters
    let store = STORE.lock().unwrap();
    let mut filtered: Vec<AnalyzedString> = store.values().cloned().collect();

    if let Some(b) = want_palindrome {
        filtered.retain(|e| e.properties.is_palindrome == b);
    }
    if let Some(n) = want_word_count {
        filtered.retain(|e| e.properties.word_count == n);
    }
    if let Some(min_len) = min_length {
        filtered.retain(|e| e.value.len() >= min_len);
    }
    if let Some(ch) = contains_character {
        filtered.retain(|e| e.value.contains(ch));
    }

    HttpResponse::Ok().json(serde_json::json!({
        "data": filtered,
        "count": filtered.len(),
        "interpreted_query": {
            "original": q,
            "parsed_filters": {
                "is_palindrome": want_palindrome,
                "word_count": want_word_count,
                "min_length": min_length,
                "contains_character": contains_character.map(|c| c.to_string())
            }
        }
    }))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/strings", web::post().to(create_string))
            .route("/strings", web::get().to(get_all))
            .route("/strings/filter-by-natural-language", web::get().to(filter_nl))
            .route("/strings/{string_value}", web::get().to(get_string))
            .route("/strings/{string_value}", web::delete().to(delete_string))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
