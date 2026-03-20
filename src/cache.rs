use std::fs;
use std::time::SystemTime;

// ─── Cache file paths ────────────────────────────────────────────────────────

const CRYPTO_CACHE: &str = "/tmp/claude-statusline-crypto-cache";
const CRYPTO_LOCK: &str = "/tmp/claude-statusline-crypto-lock";
const USAGE_CACHE: &str = "/tmp/claude-statusline-usage-cache";
const USAGE_LOCK: &str = "/tmp/claude-statusline-usage-lock";

pub fn crypto_cache_path() -> &'static str {
    CRYPTO_CACHE
}

pub fn usage_cache_path() -> &'static str {
    USAGE_CACHE
}

// ─── Cache core ──────────────────────────────────────────────────────────────

/// Read cached content and trigger a background refresh when stale.
///
/// - Returns the existing cached content immediately (stale-while-revalidate).
/// - If the cache is older than `max_age_secs` (or missing), spawns a
///   background thread that calls `fetch_fn` and writes the result.
/// - A `mkdir`-based lock prevents concurrent refresh attempts.
pub fn read_or_refresh<F>(
    cache_path: &str,
    lock_path: &str,
    max_age_secs: u64,
    fetch_fn: F,
) -> Option<String>
where
    F: FnOnce() -> Option<String> + Send + 'static,
{
    let content = fs::read_to_string(cache_path).ok();
    let age = file_age_secs(cache_path);

    if age.map_or(true, |a| a >= max_age_secs) {
        // mkdir-based lock (same pattern as the .sh version)
        if fs::create_dir(lock_path).is_ok() {
            let lock = lock_path.to_string();
            let cache = cache_path.to_string();
            std::thread::spawn(move || {
                if let Some(data) = fetch_fn() {
                    let _ = fs::write(&cache, &data);
                } else {
                    // touch to avoid immediate retry
                    let _ = fs::write(&cache, "");
                }
                let _ = fs::remove_dir(&lock);
            });
        }
    }

    content.filter(|s| !s.is_empty())
}

fn file_age_secs(path: &str) -> Option<u64> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    SystemTime::now()
        .duration_since(modified)
        .ok()
        .map(|d| d.as_secs())
}

// ─── Crypto fetcher ──────────────────────────────────────────────────────────

pub fn fetch_crypto(coins: &[String]) -> Option<String> {
    let prices: Vec<String> = coins
        .iter()
        .map(|coin| {
            let pair = format!("{}USDT", coin);
            let url = format!(
                "https://api.binance.com/api/v3/ticker/price?symbol={}",
                pair
            );
            match ureq::get(&url).call() {
                Ok(resp) => resp
                    .into_json::<serde_json::Value>()
                    .ok()
                    .and_then(|v| v["price"].as_str().map(String::from))
                    .unwrap_or_default(),
                Err(_) => String::new(),
            }
        })
        .collect();

    if prices.iter().all(|p: &String| !p.is_empty()) {
        Some(prices.join("|"))
    } else {
        None
    }
}

// ─── Usage fetcher ───────────────────────────────────────────────────────────

pub fn fetch_usage() -> Option<String> {
    let token = get_oauth_token()?;
    let resp = ureq::get("https://api.anthropic.com/api/oauth/usage")
        .set("Authorization", &format!("Bearer {}", token))
        .set("anthropic-beta", "oauth-2025-04-20")
        .set("User-Agent", "claude-statusline-config/2.0.0")
        .call()
        .ok()?;
    let json: serde_json::Value = resp.into_json().ok()?;
    let utilization = json["five_hour"]["utilization"].as_f64()?;
    let resets_at = json["five_hour"]["resets_at"].as_str().unwrap_or("");
    Some(format!("{}|{}", utilization as u64, resets_at))
}

#[cfg(target_os = "macos")]
fn get_oauth_token() -> Option<String> {
    let output = std::process::Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let creds: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    creds["claudeAiOauth"]["accessToken"]
        .as_str()
        .map(String::from)
}

#[cfg(not(target_os = "macos"))]
fn get_oauth_token() -> Option<String> {
    None // TODO: Linux credential reading
}

// ─── Integration with render pipeline ────────────────────────────────────────

/// Called from the render pipeline to ensure caches are fresh.
///
/// For each enabled segment that relies on a cache file (crypto, usage),
/// this calls `read_or_refresh` which will spawn a background fetch thread
/// when the cache is stale. The actual render functions still read the
/// cache files directly.
pub fn ensure_caches_fresh(config: &crate::config::Config) {
    let s = &config.segments;

    if s.crypto.enabled {
        let coins = s.crypto.coins.clone();
        read_or_refresh(
            CRYPTO_CACHE,
            CRYPTO_LOCK,
            s.crypto.refresh_interval,
            move || fetch_crypto(&coins),
        );
    }

    if s.usage.enabled {
        read_or_refresh(USAGE_CACHE, USAGE_LOCK, s.usage.refresh_interval, || {
            fetch_usage()
        });
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper: create a temp file with given content and return its path.
    fn temp_file(name: &str, content: &str) -> String {
        let path = format!("/tmp/claude-statusline-cache-test-{}", name);
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    /// Helper: remove a file if it exists.
    fn cleanup(path: &str) {
        let _ = fs::remove_file(path);
        let _ = fs::remove_dir(path);
    }

    #[test]
    fn test_file_age_secs() {
        let path = temp_file("age", "hello");
        let age = file_age_secs(&path).expect("should get age");
        assert!(age <= 1, "freshly created file should be 0 or 1 second old, got {}", age);
        cleanup(&path);
    }

    #[test]
    fn test_read_or_refresh_fresh_cache() {
        let cache = temp_file("fresh", "cached-data");
        let lock = format!("{}-lock", cache);
        cleanup(&lock);

        // max_age very large → cache is fresh, no refresh triggered
        let result = read_or_refresh(&cache, &lock, 9999, || {
            panic!("fetch_fn should not be called for fresh cache");
        });

        assert_eq!(result, Some("cached-data".to_string()));
        cleanup(&cache);
        cleanup(&lock);
    }

    #[test]
    fn test_read_or_refresh_stale_cache() {
        let cache = temp_file("stale", "old-data");
        let lock = format!("{}-lock", cache);
        cleanup(&lock);

        // max_age=0 → cache is always stale, background refresh triggered
        let result = read_or_refresh(&cache, &lock, 0, || Some("new-data".to_string()));

        // Should return the old cached content immediately (stale-while-revalidate)
        assert_eq!(result, Some("old-data".to_string()));

        // Wait briefly for background thread to finish
        std::thread::sleep(std::time::Duration::from_millis(100));

        // After refresh, cache file should contain new data
        let updated = fs::read_to_string(&cache).unwrap();
        assert_eq!(updated, "new-data");

        cleanup(&cache);
        cleanup(&lock);
    }

    #[test]
    fn test_read_or_refresh_no_cache() {
        let cache = "/tmp/claude-statusline-cache-test-nonexistent";
        let lock = format!("{}-lock", cache);
        cleanup(cache);
        cleanup(&lock);

        let result = read_or_refresh(cache, &lock, 60, || Some("fetched".to_string()));

        // No cached content to return
        assert!(result.is_none());

        // Wait for background thread
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Cache file should now exist with fetched data
        let written = fs::read_to_string(cache).unwrap();
        assert_eq!(written, "fetched");

        cleanup(cache);
        cleanup(&lock);
    }

    #[test]
    fn test_read_or_refresh_empty_cache() {
        let cache = temp_file("empty", "");
        let lock = format!("{}-lock", cache);
        cleanup(&lock);

        let result = read_or_refresh(&cache, &lock, 9999, || {
            panic!("should not refresh fresh cache");
        });

        // Empty content is filtered out → None
        assert!(result.is_none());

        cleanup(&cache);
        cleanup(&lock);
    }

    #[test]
    fn test_crypto_cache_path() {
        assert_eq!(crypto_cache_path(), "/tmp/claude-statusline-crypto-cache");
    }

    #[test]
    fn test_usage_cache_path() {
        assert_eq!(usage_cache_path(), "/tmp/claude-statusline-usage-cache");
    }
}
