use colored::Colorize;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use itertools::Itertools;
use passwords::PasswordGenerator;
use rquest::Proxy;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    path::PathBuf,
    str::FromStr,
};
use tiktoken_rs::o200k_base;
use tracing::{error, info, warn};

use crate::error::ClewdrError;

pub const CONFIG_NAME: &str = "config.toml";
pub const ENDPOINT: &str = "https://claude.ai";

const fn default_max_retries() -> usize {
    5
}
fn default_ip() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    8484
}
const fn default_use_real_roles() -> bool {
    true
}
const fn default_padtxt_len() -> usize {
    4000
}

/// A struct representing the configuration of the application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClewdrConfig {
    // App settings
    #[serde(default)]
    pub check_update: bool,
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default)]
    pub check_duplicate: bool,

    // Cookie configurations
    #[serde(default)]
    pub cookie_array: Vec<CookieStatus>,
    #[serde(default)]
    pub wasted_cookie: Vec<UselessCookie>,

    // Network settings
    #[serde(default)]
    password: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub rproxy: String,
    #[serde(default = "default_ip")]
    ip: String,
    #[serde(default = "default_port")]
    port: u16,

    // Api settings
    #[serde(default)]
    pub enable_oai: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    #[serde(default)]
    pub pass_params: bool,
    #[serde(default)]
    pub preserve_chats: bool,

    // Cookie settings
    #[serde(default)]
    pub skip_warning: bool,
    #[serde(default)]
    pub skip_restricted: bool,
    #[serde(default)]
    pub skip_non_pro: bool,

    // Prompt configurations
    #[serde(default = "default_use_real_roles")]
    pub use_real_roles: bool,
    #[serde(default)]
    pub custom_h: Option<String>,
    #[serde(default)]
    pub custom_a: Option<String>,
    #[serde(default)]
    pub custom_prompt: String,
    #[serde(default)]
    pub padtxt_file: String,
    #[serde(default = "default_padtxt_len")]
    pub padtxt_len: usize,

    // Skip field
    #[serde(skip)]
    pub rquest_proxy: Option<Proxy>,
    #[serde(skip)]
    pub pad_tokens: Vec<String>,
}

/// Reason why a cookie is considered useless
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Reason {
    NonPro,
    Disabled,
    Banned,
    Null,
    Restricted(i64),
    TooManyRequest(i64),
}

impl Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::Disabled => write!(f, "Organization Disabled"),
            Reason::NonPro => write!(f, "Free account"),
            Reason::Banned => write!(f, "Banned"),
            Reason::Null => write!(f, "Null"),
            Reason::Restricted(i) => {
                let time = chrono::DateTime::from_timestamp(*i, 0)
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string().yellow())
                    .unwrap_or("Invalid date".to_string().yellow());
                write!(f, "Restricted: until {}", time)
            }
            Reason::TooManyRequest(i) => {
                let time = chrono::DateTime::from_timestamp(*i, 0)
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string().yellow())
                    .unwrap_or("Invalid date".to_string().yellow());
                write!(f, "429 Too many request: until {}", time)
            }
        }
    }
}

/// A struct representing a useless cookie
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UselessCookie {
    pub cookie: ClewdrCookie,
    pub reason: Reason,
}
impl PartialEq for UselessCookie {
    fn eq(&self, other: &Self) -> bool {
        self.cookie == other.cookie
    }
}
impl Eq for UselessCookie {}
impl Hash for UselessCookie {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cookie.hash(state);
    }
}

impl UselessCookie {
    pub fn new(cookie: ClewdrCookie, reason: Reason) -> Self {
        Self { cookie, reason }
    }
}

/// A struct representing a cookie with its information
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CookieStatus {
    pub cookie: ClewdrCookie,
    #[serde(default)]
    pub reset_time: Option<i64>,
}

impl PartialEq for CookieStatus {
    fn eq(&self, other: &Self) -> bool {
        self.cookie == other.cookie
    }
}
impl Eq for CookieStatus {}
impl Hash for CookieStatus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cookie.hash(state);
    }
}
impl Ord for CookieStatus {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cookie.cmp(&other.cookie)
    }
}
impl PartialOrd for CookieStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Default cookie value for testing purposes
const PLACEHOLDER_COOKIE: &str = "sk-ant-sid01----------------------------SET_YOUR_COOKIE_HERE----------------------------------------AAAAAAAA";

impl CookieStatus {
    pub fn new(cookie: &str, reset_time: Option<i64>) -> Self {
        Self {
            cookie: ClewdrCookie::from(cookie),
            reset_time,
        }
    }

    /// check if the cookie is expired
    /// if expired, set the reset time to None
    pub fn reset(self) -> Self {
        if let Some(t) = self.reset_time {
            if t < chrono::Utc::now().timestamp() {
                info!("Cookie reset time expired");
                return Self {
                    reset_time: None,
                    ..self
                };
            }
        }
        self
    }
}

/// A struct representing a cookie
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClewdrCookie {
    inner: String,
}

impl Default for ClewdrCookie {
    fn default() -> Self {
        Self {
            inner: PLACEHOLDER_COOKIE.to_string(),
        }
    }
}

impl ClewdrCookie {
    /// Check if the cookie is valid format
    pub fn validate(&self) -> bool {
        // Check if the cookie is valid
        let re = regex::Regex::new(r"^sk-ant-sid01-[0-9A-Za-z_-]{86}-[0-9A-Za-z_-]{6}AA$").unwrap();
        re.is_match(&self.inner)
    }
}

impl From<&str> for ClewdrCookie {
    /// Create a new cookie from a string
    fn from(original: &str) -> Self {
        // split off first '@' to keep compatibility with clewd
        let cookie = original.split_once('@').map_or(original, |(_, c)| c);
        // only keep '=' '_' '-' and alphanumeric characters
        let cookie = cookie
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '=' || *c == '_' || *c == '-')
            .collect::<String>()
            .trim_start_matches("sessionKey=")
            .to_string();
        let cookie = Self { inner: cookie };
        if !cookie.validate() {
            warn!("Invalid cookie format: {}", original);
        }
        cookie
    }
}

impl Display for ClewdrCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sessionKey={}", self.inner)
    }
}

impl Debug for ClewdrCookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sessionKey={}", self.inner)
    }
}

impl Serialize for ClewdrCookie {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let str = self.to_string();
        serializer.serialize_str(&str)
    }
}

impl<'de> Deserialize<'de> for ClewdrCookie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ClewdrCookie::from(s.as_str()))
    }
}

/// Generate a random password of given length
fn generate_password() -> String {
    let pg = PasswordGenerator {
        length: 64,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        spaces: false,
        exclude_similar_characters: true,
        strict: true,
    };

    println!(
        "{}",
        "Generating random password, paste it to your proxy setting in SillyTavern".green()
    );
    pg.generate_one().unwrap()
}

impl Default for ClewdrConfig {
    fn default() -> Self {
        Self {
            enable_oai: false,
            max_retries: default_max_retries(),
            check_update: true,
            auto_update: false,
            cookie_array: vec![],
            wasted_cookie: Vec::new(),
            check_duplicate: true,
            password: String::new(),
            proxy: String::new(),
            ip: "127.0.0.1".to_string(),
            port: 8484,
            rproxy: String::new(),
            use_real_roles: true,
            custom_prompt: String::new(),
            padtxt_file: String::new(),
            padtxt_len: 4000,
            custom_h: None,
            custom_a: None,
            rquest_proxy: None,
            pad_tokens: Vec::new(),
            pass_params: false,
            preserve_chats: false,
            skip_warning: false,
            skip_restricted: false,
            skip_non_pro: false,
        }
    }
}

impl Display for ClewdrConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // one line per field
        write!(
            f,
            "Password: {}\n\
            Forward Proxy: {}\n\
            Reverse Proxy: {}\n\
            Available Cookies in array: {}\n",
            self.password.yellow(),
            self.proxy.to_string().blue(),
            self.rproxy.to_string().blue(),
            self.cookie_array
                .iter()
                .filter(|x| x.reset_time.is_none())
                .count()
                .to_string()
                .blue()
        )?;
        if !self.pad_tokens.is_empty() {
            Ok(writeln!(
                f,
                "Pad txt token count: {}",
                self.pad_tokens.len().to_string().blue()
            )?)
        } else {
            Ok(())
        }
    }
}

impl ClewdrConfig {
    pub fn auth(&self, key: &str) -> bool {
        key == self.password
    }

    /// Load the configuration from the file
    pub fn load() -> Result<Self, ClewdrError> {
        let config: ClewdrConfig = Figment::new()
            .merge(Toml::file(CONFIG_NAME))
            .merge(Env::prefixed("CLEWDR_"))
            .extract_lossy()?;
        let config = config.validate();
        config.save()?;
        Ok(config)
    }

    fn load_padtxt(&mut self) {
        let padtxt = &self.padtxt_file.trim();
        if padtxt.is_empty() {
            return;
        }
        let Ok(padtxt_path) = PathBuf::from_str(padtxt);
        if !padtxt_path.exists() {
            error!("Pad txt file not found: {}", padtxt_path.display());
            return;
        }
        let Ok(padtxt_string) = std::fs::read_to_string(padtxt_path.as_path()) else {
            error!("Failed to read pad txt file: {}", padtxt_path.display());
            return;
        };
        // remove tokenizer special characters

        let bpe = o200k_base().unwrap();
        let ranks = bpe.encode_with_special_tokens(&padtxt_string);
        let mut tokens = Vec::with_capacity(4096);
        for token in ranks {
            let Ok(token) = bpe.decode(vec![token]) else {
                continue;
            };
            tokens.push(token);
        }
        if tokens.len() < 4096 {
            panic!("Pad txt file is too short: {}", padtxt_path.display());
        }
        self.pad_tokens = tokens;
    }

    /// API endpoint of server
    pub fn endpoint(&self) -> String {
        if self.rproxy.is_empty() {
            ENDPOINT.to_string()
        } else {
            self.rproxy.clone()
        }
    }

    /// address of proxy
    pub fn address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    /// Save the configuration to a file
    pub fn save(&self) -> Result<(), ClewdrError> {
        let Ok(config_path) = PathBuf::from_str(CONFIG_NAME);
        std::fs::write(config_path, toml::ser::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Validate the configuration
    fn validate(mut self) -> Self {
        if self.password.trim().is_empty() {
            self.password = generate_password();
            self.save().expect("Failed to save config");
        }
        self.cookie_array = self
            .cookie_array
            .into_iter()
            .map(|x| x.reset())
            .sorted()
            .collect();
        self.cookie_array.dedup();
        self.ip = self.ip.trim().to_string();
        self.rproxy = self.rproxy.trim().to_string();
        self.proxy = self.proxy.trim().to_string();
        let proxy = if self.proxy.is_empty() {
            None
        } else {
            Proxy::all(self.proxy.clone())
                .inspect_err(|e| {
                    error!("Failed to parse proxy: {}", e);
                })
                .ok()
        };
        self.rquest_proxy = proxy;
        self.load_padtxt();
        self
    }
}
