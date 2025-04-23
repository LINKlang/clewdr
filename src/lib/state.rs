use colored::Colorize;
use regex::RegexBuilder;
use rquest::Response;
use rquest::header::SET_COOKIE;
use serde_json::json;
use tracing::debug;
use tracing::error;

use std::collections::HashMap;
use std::sync::Arc;

use crate::client::SUPER_CLIENT;
use crate::client::SetupRequest;
use crate::config::ClewdrConfig;
use crate::config::CookieStatus;
use crate::config::Reason;
use crate::cookie_manager::CookieEventSender;
use crate::error::ClewdrError;

/// State of current connection
#[derive(Clone)]
pub struct ClientState {
    pub cookie: Option<CookieStatus>,
    pub config: Arc<ClewdrConfig>,
    pub event_sender: CookieEventSender,
    pub org_uuid: Option<String>,
    pub conv_uuid: Option<String>,
    cookies: HashMap<String, String>,
    pub capabilities: Vec<String>,
}

impl ClientState {
    /// Create a new AppState instance
    pub fn new(config: ClewdrConfig, event_sender: CookieEventSender) -> Self {
        ClientState {
            config: Arc::new(config),
            event_sender,
            cookie: None,
            org_uuid: None,
            conv_uuid: None,
            cookies: HashMap::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn is_pro(&self) -> bool {
        self.capabilities.iter().any(|c| {
            c.contains("pro")
                || c.contains("enterprise")
                || c.contains("raven")
                || c.contains("max")
        })
    }

    /// Update cookie from the server response
    pub fn update_cookie_from_res(&mut self, res: &Response) {
        if let Some(s) = res.headers().get(SET_COOKIE).and_then(|h| h.to_str().ok()) {
            self.update_cookies(s)
        }
    }

    /// Update cookies from string
    fn update_cookies(&mut self, str: &str) {
        let str = str.split("\n").to_owned().collect::<Vec<_>>().join("");
        if str.is_empty() {
            return;
        }
        let re = RegexBuilder::new(r"^(path|expires|domain|HttpOnly|Secure|SameSite)[=;]*")
            .case_insensitive(true)
            .build()
            .unwrap();
        str.split(";")
            .filter(|s| !re.is_match(s) && !s.is_empty())
            .for_each(|s| {
                let Some((name, value)) = s.split_once("=").map(|(n, v)| (n.trim(), v.trim()))
                else {
                    return;
                };
                if name.is_empty() || value.is_empty() {
                    return;
                }
                self.cookies.insert(name.to_string(), value.to_string());
            });
    }

    /// Current cookie string that are used in requests
    pub fn header_cookie(&self) -> String {
        // check rotating guard
        self.cookies
            .iter()
            .map(|(name, value)| format!("{}={}", name, value))
            .collect::<Vec<_>>()
            .join("; ")
            .trim()
            .to_string()
    }

    /// request a new cookie from cookie manager
    pub async fn request_cookie(&mut self) -> Result<(), ClewdrError> {
        let res = self.event_sender.request().await?;
        self.cookie = Some(res.clone());
        self.update_cookies(res.cookie.to_string().as_str());
        println!("Cookie: {}", res.cookie.to_string().green());
        Ok(())
    }

    /// return the cookie to the cookie manager
    pub async fn return_cookie(&mut self, reason: Option<Reason>) {
        // return the cookie to the cookie manager
        if let Some(cookie) = self.cookie.take() {
            self.event_sender
                .return_cookie(cookie, reason)
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to send cookie: {}", e);
                });
        }
    }

    /// Delete current chat conversation
    pub async fn clean_chat(&self) -> Result<(), ClewdrError> {
        let Some(ref org_uuid) = self.org_uuid else {
            return Ok(());
        };
        let Some(ref conv_uuid) = self.conv_uuid else {
            return Ok(());
        };
        // if preserve_chats is true, do not delete chat, just rename it
        if self.config.preserve_chats {
            debug!("Renaming chat: {}", conv_uuid);
            let endpoint = format!(
                "{}/api/organizations/{}/chat_conversations/{}",
                self.config.endpoint(),
                org_uuid,
                conv_uuid
            );
            let pld = json!({
                "name": format!("ClewdR-{}-{}", org_uuid, conv_uuid),
            });
            let proxy = self.config.rquest_proxy.clone();
            let _ = SUPER_CLIENT
                .put(endpoint)
                .setup_request(conv_uuid, self.header_cookie(), proxy)
                .json(&pld)
                .send()
                .await?;
            return Ok(());
        }
        debug!("Deleting chat: {}", conv_uuid);
        let endpoint = format!(
            "{}/api/organizations/{}/chat_conversations/{}",
            self.config.endpoint(),
            org_uuid,
            conv_uuid
        );
        let proxy = self.config.rquest_proxy.clone();
        let _ = SUPER_CLIENT
            .delete(endpoint)
            .setup_request(conv_uuid, self.header_cookie(), proxy)
            .send()
            .await?;
        Ok(())
    }
}
