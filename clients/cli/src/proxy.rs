//! Proxy Management
//!
//! Handles loading and selecting random proxies from proxies.txt file

use rand::seq::SliceRandom;
use reqwest::Proxy;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Proxy configuration structure
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl ProxyConfig {
    /// Create a new proxy config from string format: host:port:username:password
    pub fn from_string(proxy_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = proxy_str.trim().split(':').collect();
        if parts.len() != 4 {
            return Err(format!("Invalid proxy format: {}", proxy_str));
        }

        let port = parts[1]
            .parse::<u16>()
            .map_err(|_| format!("Invalid port in proxy: {}", proxy_str))?;

        Ok(ProxyConfig {
            host: parts[0].to_string(),
            port,
            username: parts[2].to_string(),
            password: parts[3].to_string(),
        })
    }

    /// Convert to reqwest::Proxy
    pub fn to_reqwest_proxy(&self) -> Result<Proxy, reqwest::Error> {
        let proxy_url = format!("http://{}:{}", self.host, self.port);
        let proxy = Proxy::http(&proxy_url)?;
        Ok(proxy.basic_auth(&self.username, &self.password))
    }

    /// Get proxy as URL string for logging (without credentials)
    pub fn to_display_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Proxy manager that loads and manages proxy rotation
pub struct ProxyManager {
    proxies: Vec<ProxyConfig>,
    last_updated: Instant,
    update_interval: Duration,
}

impl ProxyManager {
    /// Create a new proxy manager
    pub fn new() -> Self {
        Self {
            proxies: Vec::new(),
            last_updated: Instant::now() - Duration::from_secs(3600), // Force initial load
            update_interval: Duration::from_secs(300), // Reload every 5 minutes
        }
    }

    /// Load proxies from file if needed (with automatic refresh)
    pub fn ensure_proxies_loaded(&mut self) -> Result<(), String> {
        if self.last_updated.elapsed() > self.update_interval || self.proxies.is_empty() {
            self.load_proxies()?;
            self.last_updated = Instant::now();
        }
        Ok(())
    }

    /// Load proxies from proxy file
    fn load_proxies(&mut self) -> Result<(), String> {
        let proxy_file_path = get_proxy_file_path();
        let proxy_file = Path::new(&proxy_file_path);
        if !proxy_file.exists() {
            return Err(format!("{} file not found", proxy_file_path));
        }

        let content = fs::read_to_string(proxy_file)
            .map_err(|e| format!("Failed to read proxies.txt: {}", e))?;

        let mut new_proxies = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            match ProxyConfig::from_string(line) {
                Ok(proxy) => new_proxies.push(proxy),
                Err(e) => {
                    eprintln!("Warning: Skipping invalid proxy on line {}: {}", line_num + 1, e);
                }
            }
        }

        if new_proxies.is_empty() {
            return Err("No valid proxies found in proxies.txt".to_string());
        }

        self.proxies = new_proxies;
        println!("Loaded {} proxies from {}", self.proxies.len(), proxy_file_path);
        Ok(())
    }

    /// Get a random proxy
    pub fn get_random_proxy(&mut self) -> Result<ProxyConfig, String> {
        self.ensure_proxies_loaded()?;
        
        if self.proxies.is_empty() {
            return Err("No proxies available".to_string());
        }

        let mut rng = rand::thread_rng();
        self.proxies
            .choose(&mut rng)
            .cloned()
            .ok_or_else(|| "Failed to select random proxy".to_string())
    }

    /// Get proxy count
    pub fn proxy_count(&self) -> usize {
        self.proxies.len()
    }
}

/// Global proxy manager instance
static PROXY_MANAGER: OnceLock<std::sync::Mutex<ProxyManager>> = OnceLock::new();

/// Global proxy enabled setting
static PROXY_ENABLED: OnceLock<std::sync::Mutex<bool>> = OnceLock::new();

/// Global proxy file path setting
static PROXY_FILE_PATH: OnceLock<std::sync::Mutex<String>> = OnceLock::new();

/// Get or initialize the global proxy manager
pub fn get_proxy_manager() -> &'static std::sync::Mutex<ProxyManager> {
    PROXY_MANAGER.get_or_init(|| std::sync::Mutex::new(ProxyManager::new()))
}

/// Get a random proxy from the global manager
pub fn get_random_proxy() -> Result<ProxyConfig, String> {
    let manager = get_proxy_manager();
    let mut manager = manager.lock().map_err(|_| "Failed to lock proxy manager")?;
    manager.get_random_proxy()
}

/// Set whether proxy should be enabled globally
pub fn set_proxy_enabled(enabled: bool) {
    let setting = PROXY_ENABLED.get_or_init(|| std::sync::Mutex::new(true));
    if let Ok(mut setting) = setting.lock() {
        *setting = enabled;
    }
}

/// Check if proxy is enabled
pub fn is_proxy_enabled() -> bool {
    let setting = PROXY_ENABLED.get_or_init(|| std::sync::Mutex::new(true));
    setting.lock().map(|s| *s).unwrap_or(true)
}

/// Set custom proxy file path
pub fn set_proxy_file_path(path: String) {
    let setting = PROXY_FILE_PATH.get_or_init(|| std::sync::Mutex::new("proxies.txt".to_string()));
    if let Ok(mut setting) = setting.lock() {
        *setting = path;
    }
}

/// Get the proxy file path (default: proxies.txt)
pub fn get_proxy_file_path() -> String {
    let setting = PROXY_FILE_PATH.get_or_init(|| std::sync::Mutex::new("proxies.txt".to_string()));
    setting.lock().map(|s| s.clone()).unwrap_or_else(|_| "proxies.txt".to_string())
}

/// Check if proxy file exists
pub fn proxy_file_exists() -> bool {
    let proxy_file_path = get_proxy_file_path();
    Path::new(&proxy_file_path).exists()
}

/// Check if proxy should be used (enabled and file exists)
pub fn should_use_proxy() -> bool {
    is_proxy_enabled() && proxy_file_exists()
} 