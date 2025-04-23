use colored::Colorize;
use rquest::Client;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{BufReader, copy};
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;
use zip::ZipArchive;

use crate::utils::STATIC_DIR;
use crate::{Args, config::ClewdrConfig, error::ClewdrError, utils::copy_dir_all};

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub struct ClewdrUpdater {
    config: ClewdrConfig,
    client: Client,
    user_agent: String,
    repo_owner: &'static str,
    repo_name: &'static str,
}

impl ClewdrUpdater {
    pub fn new(config: ClewdrConfig) -> Result<Self, ClewdrError> {
        let authors = option_env!("CARGO_PKG_AUTHORS").unwrap_or_default();
        let repo_owner = authors.split(':').next().unwrap_or("Xerxes-2");
        let repo_name = env!("CARGO_PKG_NAME");
        let policy = rquest::redirect::Policy::default();
        let client = rquest::Client::builder().redirect(policy).build()?;

        let user_agent = format!(
            "clewdr/{} (+https://github.com/{}/{})",
            env!("CARGO_PKG_VERSION"),
            repo_owner,
            repo_name
        );

        Ok(Self {
            config,
            client,
            user_agent,
            repo_owner,
            repo_name,
        })
    }

    pub async fn check_for_updates(&self) -> Result<bool, ClewdrError> {
        let args: Args = clap::Parser::parse();
        if !args.update && !self.config.check_update {
            return Ok(false);
        }

        info!("Checking for updates...");
        // info!("User-Agent: {}", self.user_agent);

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?
            .error_for_status()?;

        let release: GitHubRelease = response.json().await?;
        let latest_version = release.tag_name.trim_start_matches('v');
        let current_version = env!("CARGO_PKG_VERSION");

        let update_available = self.compare_versions(current_version, latest_version)?;

        if !update_available {
            info!("Already at the latest version {}", current_version.green());
            return Ok(false);
        }
        info!(
            "New version {} available (current: {})",
            latest_version.green().italic(),
            current_version.yellow()
        );
        // Auto update if enabled
        if args.update || self.config.auto_update {
            self.perform_update(&release).await?;
        }

        Ok(true)
    }

    async fn perform_update(&self, release: &GitHubRelease) -> Result<(), ClewdrError> {
        let latest_version = release.tag_name.trim_start_matches('v');

        // Find appropriate asset for this platform
        let asset = self.find_appropriate_asset(release)?;

        info!("Downloading update from {}", asset.browser_download_url);

        // Create a temporary directory
        let temp_dir = tempfile::tempdir()?;
        let zip_path = temp_dir.path().join("update.zip");

        // Download the asset
        let response = self
            .client
            .get(&asset.browser_download_url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await?
            .error_for_status()?;

        // Save the downloaded file
        let content = response.bytes().await?;
        let mut file = File::create(&zip_path)?;
        copy(&mut content.as_ref(), &mut file)?;

        // Extract the zip
        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir_all(&extract_dir)?;

        let file = File::open(&zip_path)?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader)?;

        // Extract all files
        archive.extract(&extract_dir)?;

        let binary_name = if cfg!(windows) {
            "clewdr.exe"
        } else {
            "clewdr"
        };
        let binary_path = extract_dir.join(binary_name);

        if !binary_path.exists() {
            return Err(ClewdrError::AssetError(format!(
                "Binary not found in the update package: {}",
                binary_name
            )));
        }

        let extract_static_path = extract_dir.join(STATIC_DIR);
        if !extract_static_path.exists() {
            return Err(ClewdrError::AssetError(
                "Static assets not found in the update package".to_string(),
            ));
        }

        // delete old static assets
        let Ok(static_path) = PathBuf::from_str(STATIC_DIR);
        if static_path.exists() {
            std::fs::remove_dir_all(&static_path)?;
        }
        // copy new static assets
        copy_dir_all(&extract_static_path, &static_path)?;
        info!("Replace new static assets to {}", static_path.display());

        // Make the binary executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }

        #[cfg(target_os = "android")]
        {
            use tracing::warn;
            let so_path = extract_dir.join("libc++_shared.so");
            if so_path.exists() {
                let current_dir = env::current_exe()?
                    .parent()
                    .ok_or(ClewdrError::PathNotFound(
                        "Failed to get current directory".to_string(),
                    ))?
                    .to_path_buf();
                let target_so_path = current_dir.join("libc++_shared.so");
                std::fs::copy(&so_path, &target_so_path)?;
                info!("Copied libc++_shared.so to the application directory");
            } else {
                warn!("libc++_shared.so not found in the update package");
            }
        }

        // Replace the current binary
        self_replace::self_replace(&binary_path)?;

        println!("Successfully updated to version {}", latest_version.green());
        println!("{}", "Update complete, closing...".green());
        std::process::exit(0);
    }

    fn find_appropriate_asset<'a>(
        &self,
        release: &'a GitHubRelease,
    ) -> Result<&'a GitHubAsset, ClewdrError> {
        // Determine platform and architecture
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        let target = match (os, arch) {
            ("windows", "x86_64") => "windows-x86_64",
            ("linux", "x86_64") => {
                if cfg!(target_env = "musl") {
                    "musllinux-x86_64"
                } else {
                    "linux-x86_64"
                }
            }
            ("linux", "aarch64") => {
                if cfg!(target_env = "musl") {
                    "musllinux-aarch64"
                } else {
                    "linux-aarch64"
                }
            }
            ("macos", "x86_64") => "macos-x86_64",
            ("macos", "aarch64") => "macos-aarch64",
            ("android", "aarch64") => "android-aarch64",
            _ => {
                return Err(ClewdrError::AssetError(format!(
                    "Unsupported platform: {}-{}",
                    os, arch
                )));
            }
        };
        info!("Detected platform: {}", target);
        release
            .assets
            .iter()
            .find(|asset| asset.name.contains(target) && asset.name.ends_with(".zip"))
            .ok_or(ClewdrError::AssetError(format!(
                "No suitable asset found for platform: {}",
                target
            )))
    }

    fn compare_versions(&self, current: &str, latest: &str) -> Result<bool, ClewdrError> {
        let parse_version = |v: &str| -> Result<(u32, u32, u32), ClewdrError> {
            let parts: Vec<&str> = v.split('.').collect();
            if parts.len() < 3 {
                return Err(ClewdrError::InvalidVersion(v.to_string()));
            }
            let major = parts[0].parse::<u32>()?;
            let minor = parts[1].parse::<u32>()?;
            let patch = parts[2].parse::<u32>()?;
            Ok((major, minor, patch))
        };

        let current = parse_version(current)?;
        let latest = parse_version(latest)?;

        Ok(current < latest)
    }
}
