use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ReleaseSource {
    pub repository: String,
    pub branch: String,
    pub commit: String,
}

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub version: u32,
    pub tag: String,
    pub framework: ReleaseSource,
    pub react_ui: ReleaseSource,
    pub timestamp: String,
    pub timestamp_unix: i64,
}

impl ReleaseInfo {
    pub fn new(tag: String, framework: ReleaseSource, react_ui: ReleaseSource) -> Self {
        let now = Utc::now();

        Self {
            version: 1,
            tag,
            framework,
            react_ui,
            timestamp: now.to_rfc3339_opts(SecondsFormat::Secs, true),
            timestamp_unix: now.timestamp(),
        }
    }

    pub fn render(&self) -> String {
        format!(
            "version={}
tag={}

framework_repository={}
framework_branch={}
framework_commit={}

react_ui_repository={}
react_ui_branch={}
react_ui_commit={}

timestamp={}
timestamp_unix={}
",
            self.version,
            self.tag,
            self.framework.repository,
            self.framework.branch,
            self.framework.commit,
            self.react_ui.repository,
            self.react_ui.branch,
            self.react_ui.commit,
            self.timestamp,
            self.timestamp_unix,
        )
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        fs::write(path, self.render())
            .with_context(|| format!("failed to write {}", path.display()))
    }

    pub fn read(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self> {
        if content.contains('\r') {
            bail!("release metadata contains CRLF line endings");
        }

        let mut values = BTreeMap::new();

        for (line_number, line) in content.lines().enumerate() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                bail!(
                    "invalid release metadata at line {}: {}",
                    line_number + 1,
                    line
                );
            };

            if key.is_empty() {
                bail!("release metadata contains an empty key");
            }

            if values.insert(key.to_owned(), value.to_owned()).is_some() {
                bail!("duplicate release metadata key: {key}");
            }
        }

        let version = parse_u32(&values, "version")?;
        let tag = required(&values, "tag")?;

        let framework = ReleaseSource {
            repository: required(&values, "framework_repository")?,
            branch: required(&values, "framework_branch")?,
            commit: required(&values, "framework_commit")?,
        };

        let react_ui = ReleaseSource {
            repository: required(&values, "react_ui_repository")?,
            branch: required(&values, "react_ui_branch")?,
            commit: required(&values, "react_ui_commit")?,
        };

        let timestamp = required(&values, "timestamp")?;
        let timestamp_unix = parse_i64(&values, "timestamp_unix")?;

        let info = Self {
            version,
            tag,
            framework,
            react_ui,
            timestamp,
            timestamp_unix,
        };

        info.validate()?;

        Ok(info)
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            bail!("unsupported release metadata version: {}", self.version);
        }

        if !self.tag.starts_with('v') || self.tag.len() <= 1 {
            bail!("invalid release tag: {}", self.tag);
        }

        validate_sha1(&self.framework.commit, "framework_commit")?;
        validate_sha1(&self.react_ui.commit, "react_ui_commit")?;

        if self.framework.repository.is_empty() {
            bail!("framework_repository is empty");
        }

        if self.framework.branch.is_empty() {
            bail!("framework_branch is empty");
        }

        if self.react_ui.repository.is_empty() {
            bail!("react_ui_repository is empty");
        }

        if self.react_ui.branch.is_empty() {
            bail!("react_ui_branch is empty");
        }

        if self.timestamp.is_empty() {
            bail!("timestamp is empty");
        }

        Ok(())
    }
}

pub fn release_dir(root: &Path) -> PathBuf {
    root.join(".release")
}

pub fn info_path(root: &Path) -> PathBuf {
    release_dir(root).join("info")
}

pub fn signature_path(root: &Path) -> PathBuf {
    release_dir(root).join("info.sig")
}

pub fn public_key_path(root: &Path) -> PathBuf {
    release_dir(root).join("pubkey")
}

fn required(values: &BTreeMap<String, String>, key: &str) -> Result<String> {
    let value = values
        .get(key)
        .cloned()
        .with_context(|| format!("missing release metadata key: {key}"))?;

    if value.is_empty() {
        bail!("release metadata key is empty: {key}");
    }

    Ok(value)
}

fn parse_u32(values: &BTreeMap<String, String>, key: &str) -> Result<u32> {
    required(values, key)?
        .parse()
        .with_context(|| format!("invalid integer value for {key}"))
}

fn parse_i64(values: &BTreeMap<String, String>, key: &str) -> Result<i64> {
    required(values, key)?
        .parse()
        .with_context(|| format!("invalid integer value for {key}"))
}

fn validate_sha1(value: &str, field: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid SHA-1 commit in {field}: {value}");
    }

    Ok(())
}
