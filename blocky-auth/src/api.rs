use std::net::{AddrParseError, Ipv4Addr};

use sha1::{Digest, Sha1};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("request error: {0:?}")]
    RequestError(#[from] reqwest::Error),
    #[error("invalid address: {0}")]
    InvalidAddr(#[from] AddrParseError),
}

pub struct BlockedServers {
    hashes: Vec<String>,
}

impl BlockedServers {
    pub async fn load() -> Result<Self, ApiError> {
        let response = reqwest::get("https://sessionserver.mojang.com/blockedservers").await?;
        let hashes = response
            .text()
            .await?
            .trim()
            .split("\n")
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        Ok(Self { hashes })
    }

    pub fn hashes(&self) -> &Vec<String> {
        &self.hashes
    }

    pub fn contains(&self, s: &str) -> bool {
        let mut hasher = Sha1::new();
        hasher.update(s);

        let hash = hasher.finalize();
        let hash = hex::encode(hash);

        self.hashes.contains(&hash)
    }

    pub fn is_domain_blocked(&self, domain: &str) -> bool {
        if domain.is_empty() {
            return false;
        }

        let domain = domain.to_lowercase();
        if self.contains(&domain) {
            return true;
        }

        let parts = domain.split(".").collect::<Vec<_>>();
        for i in 0..parts.len() {
            let domain = format!("*.{}", parts[i..].join("."));
            if self.contains(&domain) {
                return true;
            }
        }

        false
    }

    pub fn is_addr_blocked(&self, addr: Ipv4Addr) -> bool {
        self.contains(&addr.to_string())
            || self.contains(&format!(
                "{}.{}.{}.*",
                addr.octets()[0],
                addr.octets()[1],
                addr.octets()[2]
            ))
            || self.contains(&format!("{}.{}.*", addr.octets()[0], addr.octets()[1]))
            || self.contains(&format!("{}.*", addr.octets()[0]))
    }

    pub fn is_blocked(&self, s: &str) -> bool {
        match s.parse() {
            Ok(addr) => self.is_addr_blocked(addr),
            Err(_) => self.is_domain_blocked(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blocked_servers() {
        let blocked_servers = BlockedServers::load()
            .await
            .expect("failed to load blocked servers");

        // positive test cases
        assert!(blocked_servers.is_domain_blocked("arkhamnetwork.org"));
        assert!(blocked_servers.is_domain_blocked("insanenetwork.org"));
        assert!(blocked_servers.is_domain_blocked("herowars.org"));
        assert!(blocked_servers.is_domain_blocked("nested.goatpvp.com"));

        // negative test cases
        assert!(!blocked_servers.is_domain_blocked("google.com"));
        assert!(!blocked_servers.is_domain_blocked("minecraft.net"));
    }
}
