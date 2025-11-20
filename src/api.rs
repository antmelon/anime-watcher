//! API client for interacting with the AllAnime service.
//!
//! This module provides functions for searching shows, fetching episode lists,
//! and retrieving stream sources from the AllAnime GraphQL API.

use crate::types::{Episode, RawShow, Show, StreamSource};
use log::{debug, info, warn};
use regex::Regex;
use serde::Deserialize;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Maximum number of retry attempts for failed requests.
const MAX_RETRIES: u32 = 3;

/// Base delay between retries in milliseconds (doubles each retry).
const BASE_RETRY_DELAY_MS: u64 = 500;

const API_URL: &str = "https://api.allanime.day/api";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

/// Stream provider types from AllAnime.
///
/// Providers are prioritized by quality and reliability for streaming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Mp4,
    Sw,
    Ok,
    Vg,
    FmHls,
    SsHls,
    Default,
    LufMp4,
    SMp4,
    Kir,
    Sak,
    Unknown(String),
}

impl Provider {
    /// Parse a provider name string into a Provider enum.
    pub fn from_name(name: &str) -> Self {
        match name {
            "Mp4" => Provider::Mp4,
            "Sw" => Provider::Sw,
            "Ok" => Provider::Ok,
            "Vg" => Provider::Vg,
            "Fm-Hls" => Provider::FmHls,
            "Ss-Hls" => Provider::SsHls,
            "Default" => Provider::Default,
            "Luf-mp4" => Provider::LufMp4,
            "S-mp4" => Provider::SMp4,
            "Kir" => Provider::Kir,
            "Sak" => Provider::Sak,
            other => Provider::Unknown(other.to_string()),
        }
    }

    /// Get the priority of this provider (lower is better).
    pub fn priority(&self) -> usize {
        match self {
            Provider::Mp4 => 0,
            Provider::Sw => 1,
            Provider::Ok => 2,
            Provider::Vg => 3,
            Provider::FmHls => 4,
            Provider::SsHls => 5,
            Provider::Default => 6,
            Provider::LufMp4 => 7,
            Provider::SMp4 => 8,
            Provider::Kir => 9,
            Provider::Sak => 10,
            Provider::Unknown(_) => 999,
        }
    }
}

/// Check if an error is retryable (network errors, timeouts, server errors).
fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout()
        || error.is_connect()
        || error.is_request()
        || error.status().map(|s| s.is_server_error()).unwrap_or(false)
}

/// Retry an async operation with exponential backoff.
///
/// Retries the operation up to `MAX_RETRIES` times on retryable errors,
/// with exponential backoff starting at `BASE_RETRY_DELAY_MS`.
///
/// # Arguments
///
/// * `operation_name` - Name of the operation for error messages
/// * `f` - Async function to retry
///
/// # Returns
///
/// The result of the operation, or the last error if all retries fail.
async fn retry_with_backoff<T, F, Fut>(
    operation_name: &str,
    f: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, reqwest::Error>>,
{
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    info!(
                        "{} succeeded after {} attempts",
                        operation_name,
                        attempt + 1
                    );
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt < MAX_RETRIES && is_retryable_error(&e) {
                    let delay = Duration::from_millis(BASE_RETRY_DELAY_MS * 2_u64.pow(attempt));
                    warn!(
                        "{} failed (attempt {}/{}): {}. Retrying in {:?}...",
                        operation_name,
                        attempt + 1,
                        MAX_RETRIES + 1,
                        e,
                        delay
                    );
                    sleep(delay).await;
                    last_error = Some(e);
                } else {
                    return Err(format!("{} failed: {}", operation_name, e).into());
                }
            }
        }
    }

    Err(format!(
        "{} failed after {} attempts: {}",
        operation_name,
        MAX_RETRIES + 1,
        last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "unknown error".to_string())
    )
    .into())
}

// Response types for shows search
#[derive(Debug, Deserialize)]
struct ShowsResponse {
    data: ShowsData,
}

#[derive(Debug, Deserialize)]
struct ShowsData {
    shows: ShowsEdges,
}

#[derive(Debug, Deserialize)]
struct ShowsEdges {
    edges: Vec<RawShow>,
}

// Response types for episodes
#[derive(Debug, Deserialize)]
pub struct EpisodeResponse {
    pub data: EpisodeData,
}

#[derive(Debug, Deserialize)]
pub struct EpisodeData {
    pub show: EpisodeShow,
}

#[derive(Debug, Deserialize)]
pub struct EpisodeShow {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "availableEpisodesDetail")]
    pub available_episodes_detail: std::collections::HashMap<String, Vec<String>>,
}

// Response types for clock.json
#[derive(Debug, Deserialize)]
struct ClockResponse {
    success: bool,
    #[serde(default)]
    links: Vec<ClockLink>,
}

#[derive(Debug, Deserialize)]
struct ClockLink {
    #[serde(default, rename = "resolutionStr")]
    resolution: Option<String>,
    #[serde(default)]
    link: Option<String>,
    #[serde(default)]
    hls: Option<String>,
}

/// Decode AllAnime's hex-encoded URLs.
///
/// AllAnime encodes some URLs using a custom hex encoding scheme where:
/// - Each character is converted to a 2-digit hex value
/// - Values less than 33 are shifted by subtracting 51 during encoding
///
/// This function reverses that process.
///
/// # Arguments
///
/// * `encoded` - The hex-encoded string, optionally prefixed with `--`
///
/// # Returns
///
/// The decoded URL string.
///
/// # Examples
///
/// ```
/// use anime_watcher::api::decode_allanime_url;
///
/// let decoded = decode_allanime_url("48656c6c6f");
/// assert_eq!(decoded, "Hello");
/// ```
pub fn decode_allanime_url(encoded: &str) -> String {
    let cleaned = encoded.trim_start_matches('-');
    let chars: Vec<char> = cleaned.chars().collect();
    let mut result = String::new();

    let mut i = 0;
    while i + 1 < chars.len() {
        let hex_pair: String = chars[i..i + 2].iter().collect();
        if let Ok(byte_val) = u8::from_str_radix(&hex_pair, 16) {
            let actual_char = if byte_val < 33 {
                (byte_val + 51) as char
            } else {
                byte_val as char
            };
            result.push(actual_char);
        }
        i += 2;
    }

    result
}

/// Extract the clock ID from an encoded AllAnime URL.
///
/// Decodes the URL and searches for an `id` query parameter.
///
/// # Arguments
///
/// * `raw` - The raw encoded URL string
///
/// # Returns
///
/// The extracted ID if found, or `None` if no ID parameter exists.
fn extract_clock_id(raw: &str) -> Option<String> {
    let decoded = decode_allanime_url(raw);
    let re = Regex::new(r"id=([^&]+)").unwrap();
    re.captures(&decoded).map(|caps| caps[1].to_string())
}

/// Search for anime shows by query.
///
/// Queries the AllAnime GraphQL API for shows matching the search term.
///
/// # Arguments
///
/// * `query` - The search term
/// * `mode` - Translation mode: "sub" for subtitled, "dub" for dubbed
///
/// # Returns
///
/// A vector of matching shows, or an error if the request fails.
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let shows = anime_watcher::api::search_shows("demon slayer", "sub").await?;
/// for show in shows {
///     println!("{}", show.to_display());
/// }
/// # Ok(())
/// # }
/// ```
pub async fn search_shows(
    query: &str,
    mode: &str,
) -> Result<Vec<Show>, Box<dyn std::error::Error>> {
    debug!("Searching for '{}' in {} mode", query, mode);

    let variables = serde_json::json!({
        "search": {
            "allowAdult": true,
            "allowUnknown": false,
            "query": query
        },
        "limit": 40,
        "page": 1,
        "translationType": mode,
        "countryOrigin": "ALL"
    });

    let query_str = r#"query ($search: SearchInput, $limit: Int, $page: Int, $translationType: VaildTranslationTypeEnumType, $countryOrigin: VaildCountryOriginEnumType) {
        shows(search: $search, limit: $limit, page: $page, translationType: $translationType, countryOrigin: $countryOrigin) {
            edges { _id name availableEpisodes __typename }
        }
    }"#;

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()?;

    let variables_str = serde_json::to_string(&variables)?;
    let query_string = query_str.to_string();

    let resp = retry_with_backoff(&format!("Search for '{}'", query), || {
        let client = client.clone();
        let variables_str = variables_str.clone();
        let query_string = query_string.clone();
        async move {
            client
                .get(API_URL)
                .header("Referer", "https://allmanga.to")
                .query(&[("variables", variables_str), ("query", query_string)])
                .send()
                .await
        }
    })
    .await?;

    let parsed: ShowsResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse search results for '{}': {}", query, e))?;

    let shows: Vec<Show> = parsed
        .data
        .shows
        .edges
        .into_iter()
        .map(|raw| {
            let count = raw.available_episodes.get(mode).copied().unwrap_or(0);
            Show {
                id: raw.id,
                name: raw.name,
                available_episodes: count,
            }
        })
        .collect();

    debug!("Found {} shows for query '{}'", shows.len(), query);

    Ok(shows)
}

/// Fetch available episodes for a show.
///
/// Retrieves the list of episode numbers available for a given show and translation mode.
///
/// # Arguments
///
/// * `show_id` - The unique identifier of the show
/// * `mode` - Translation mode: "sub" for subtitled, "dub" for dubbed
///
/// # Returns
///
/// A vector of episodes, or an error if the request fails.
pub async fn fetch_episodes(
    show_id: &str,
    mode: &str,
) -> Result<Vec<Episode>, Box<dyn std::error::Error>> {
    debug!("Fetching episodes for show {} in {} mode", show_id, mode);

    let variables = serde_json::json!({
        "showId": show_id,
    });

    const EPISODES_QUERY: &str = r#"
        query ($showId: String!) {
            show(_id: $showId) {
                _id
                availableEpisodesDetail
            }
        }
    "#;

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()?;

    let variables_str = serde_json::to_string(&variables)?;
    let query_string = EPISODES_QUERY.to_string();

    let resp = retry_with_backoff("Fetch episodes", || {
        let client = client.clone();
        let variables_str = variables_str.clone();
        let query_string = query_string.clone();
        async move {
            client
                .get(API_URL)
                .header("Referer", "https://allmanga.to")
                .query(&[("variables", variables_str), ("query", query_string)])
                .send()
                .await
        }
    })
    .await?;

    let parsed: EpisodeResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse episode list: {}", e))?;

    let episode_list = parsed
        .data
        .show
        .available_episodes_detail
        .get(mode)
        .cloned()
        .unwrap_or_default();

    let episodes: Vec<Episode> = episode_list
        .into_iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .map(|num| Episode {
            id: format!("{}-{}", parsed.data.show.id, num),
            number: num,
            title: None,
        })
        .collect();

    debug!("Found {} episodes for show {}", episodes.len(), show_id);

    Ok(episodes)
}

/// Fetch stream sources for a specific episode.
///
/// Retrieves available streaming URLs for an episode from various providers.
/// Sources are sorted by preference, with direct URL providers prioritized.
///
/// # Arguments
///
/// * `show_id` - The unique identifier of the show
/// * `mode` - Translation mode: "sub" for subtitled, "dub" for dubbed
/// * `episode_str` - The episode number as a string (e.g., "1", "12")
///
/// # Returns
///
/// A vector of stream sources, or an error if the request fails.
/// Returns an empty vector if no sources are found.
pub async fn fetch_stream_sources(
    show_id: &str,
    mode: &str,
    episode_str: &str,
) -> Result<Vec<StreamSource>, Box<dyn std::error::Error>> {
    debug!(
        "Fetching stream sources for episode {} of show {}",
        episode_str, show_id
    );

    let variables = serde_json::json!({
        "showId": show_id,
        "translationType": mode,
        "episodeString": episode_str,
    });

    let query_str = r#"
        query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
            episode(
                showId: $showId
                translationType: $translationType
                episodeString: $episodeString
            ) {
                episodeString
                sourceUrls
            }
        }
    "#;

    #[derive(Debug, Deserialize)]
    struct EpisodeSourcesResponse {
        data: EpisodeSourcesData,
    }

    #[derive(Debug, Deserialize)]
    struct EpisodeSourcesData {
        episode: EpisodeSourcesEpisode,
    }

    #[derive(Debug, Deserialize)]
    struct EpisodeSourcesEpisode {
        #[serde(rename = "sourceUrls")]
        source_urls: Vec<SourceUrlEntry>,
    }

    #[derive(Debug, Clone, Deserialize)]
    struct SourceUrlEntry {
        #[serde(rename = "sourceUrl")]
        source_url: String,
        #[serde(rename = "sourceName")]
        source_name: String,
    }

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .timeout(Duration::from_secs(30))
        .build()?;

    let variables_str = serde_json::to_string(&variables)?;
    let query_string = query_str.to_string();

    let resp = retry_with_backoff(
        &format!("Fetch sources for episode {}", episode_str),
        || {
            let client = client.clone();
            let variables_str = variables_str.clone();
            let query_string = query_string.clone();
            async move {
                client
                    .get(API_URL)
                    .header("Referer", "https://allmanga.to")
                    .query(&[("variables", variables_str), ("query", query_string)])
                    .send()
                    .await
            }
        },
    )
    .await?;

    let parsed: EpisodeSourcesResponse = resp.json().await.map_err(|e| {
        format!(
            "Failed to parse stream sources for episode {}: {}",
            episode_str, e
        )
    })?;

    if parsed.data.episode.source_urls.is_empty() {
        return Ok(vec![]);
    }

    // Sort sources by provider priority
    let mut sorted_sources = parsed.data.episode.source_urls.clone();
    sorted_sources.sort_by(|a, b| {
        let a_provider = Provider::from_name(&a.source_name);
        let b_provider = Provider::from_name(&b.source_name);
        a_provider.priority().cmp(&b_provider.priority())
    });

    let mut result = Vec::new();

    for source in &sorted_sources {
        // Handle regular URLs (not hex-encoded)
        if source.source_url.starts_with("http") || source.source_url.starts_with("//") {
            let url = if source.source_url.starts_with("//") {
                format!("https:{}", source.source_url)
            } else {
                source.source_url.clone()
            };
            result.push(StreamSource { quality: 0, url });
            continue;
        }

        // Handle hex-encoded URLs
        if !source.source_url.starts_with("--") {
            continue;
        }

        let decoded_url = decode_allanime_url(&source.source_url);

        // Check if decoded URL is a direct video URL
        if decoded_url.starts_with("http") {
            result.push(StreamSource {
                quality: 0,
                url: decoded_url,
            });
            continue;
        }

        // Try clock.json endpoint for encoded sources
        if let Some(clock_id) = extract_clock_id(&source.source_url) {
            let clock_url = format!("https://allanime.day/apivtwo/clock.json?id={clock_id}");

            if let Ok(clock_resp) = client
                .get(&clock_url)
                .header("Referer", "https://allanime.day")
                .send()
                .await
            {
                if let Ok(clock_json) = clock_resp.json::<ClockResponse>().await {
                    if clock_json.success {
                        for link in clock_json.links {
                            if let Some(url) = link.link {
                                let quality = link
                                    .resolution
                                    .as_deref()
                                    .unwrap_or("0")
                                    .parse()
                                    .unwrap_or(0);
                                result.push(StreamSource { quality, url });
                            }

                            if let Some(hls_url) = link.hls {
                                result.push(StreamSource {
                                    quality: 0,
                                    url: hls_url,
                                });
                            }
                        }

                        if !result.is_empty() {
                            break;
                        }
                    }
                }
            }
        }
    }

    debug!(
        "Found {} stream sources for episode {}",
        result.len(),
        episode_str
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_allanime_url_empty() {
        assert_eq!(decode_allanime_url(""), "");
        assert_eq!(decode_allanime_url("--"), "");
    }

    #[test]
    fn test_decode_allanime_url_strips_dashes() {
        // Test that leading dashes are stripped
        let with_dashes = "--48656c6c6f";
        let without_dashes = "48656c6c6f";
        assert_eq!(
            decode_allanime_url(with_dashes),
            decode_allanime_url(without_dashes)
        );
    }

    #[test]
    fn test_decode_allanime_url_basic_hex() {
        // "Hello" in hex where all chars >= 33
        // H=72=0x48, e=101=0x65, l=108=0x6c, l=108=0x6c, o=111=0x6f
        let encoded = "48656c6c6f";
        assert_eq!(decode_allanime_url(encoded), "Hello");
    }

    #[test]
    fn test_decode_allanime_url_with_shift() {
        // Test values < 33 that need +51 shift
        // Character 'D' (68) encoded as 68-51=17 -> 0x11
        // Character 'C' (67) encoded as 67-51=16 -> 0x10
        let encoded = "1110"; // Should decode to "DC"
        let decoded = decode_allanime_url(encoded);
        assert_eq!(decoded, "DC");
    }

    #[test]
    fn test_decode_allanime_url_odd_length() {
        // Odd length strings should just skip the last character
        let encoded = "48656c6c6"; // "Hello" with last char truncated
        assert_eq!(decode_allanime_url(encoded), "Hell");
    }

    #[test]
    fn test_decode_allanime_url_invalid_hex() {
        // Invalid hex characters should be skipped
        let encoded = "48XX6c"; // H, invalid, l
        let decoded = decode_allanime_url(encoded);
        assert_eq!(decoded, "Hl");
    }

    #[test]
    fn test_extract_clock_id_with_id() {
        // Create a URL with id parameter, encode it
        // "?id=abc123&other=val" -> encode each char
        // For simplicity, test with raw decoded URL check
        // The function decodes then extracts, so we need properly encoded input

        // Test with a simple case - character '?' is 63, 'i' is 105, 'd' is 100, '=' is 61
        // 63=0x3f, 105=0x69, 100=0x64, 61=0x3d
        // "?id=test" -> 3f69643d74657374
        let encoded = "--3f69643d74657374";
        let result = extract_clock_id(encoded);
        assert_eq!(result, Some("test".to_string()));
    }

    #[test]
    fn test_extract_clock_id_no_id() {
        // Encode "?other=value" - no id parameter
        // '?'=63=0x3f, 'o'=111=0x6f, etc.
        let encoded = "--3f6f746865723d76616c7565";
        let result = extract_clock_id(encoded);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_clock_id_empty() {
        let result = extract_clock_id("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_clock_id_with_ampersand() {
        // "?id=abc&other=123" - should extract only "abc"
        // Encoded: 3f69643d61626326...
        let encoded = "--3f69643d616263266f746865723d313233";
        let result = extract_clock_id(encoded);
        assert_eq!(result, Some("abc".to_string()));
    }
}
