use serde::{
    Deserialize,
    Serialize,
};

// Structs for the latest github release API response
// https://api.github.com/repos/:owner/:repo/releases/latest
#[derive(Serialize, Deserialize, Debug)]
pub struct GithubRelease {
    url: String,
    assets_url: String,
    upload_url: String,
    html_url: String,
    id: i64,
    node_id: String,
    pub(crate) tag_name: String,
    target_commitish: String,
    name: Option<serde_json::Value>,
    draft: bool,
    author: Author,
    prerelease: bool,
    created_at: String,
    published_at: String,
    pub(crate) assets: Vec<Asset>,
    tarball_url: String,
    zipball_url: String,
    body: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    url: String,
    id: i64,
    node_id: String,
    name: String,
    label: String,
    uploader: Author,
    content_type: String,
    state: String,
    size: i64,
    download_count: i64,
    created_at: String,
    updated_at: String,
    pub(crate) browser_download_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Author {
    login: String,
    id: i64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(rename = "type")]
    author_type: String,
    site_admin: bool,
}

pub type GithubTag = Vec<GithubTagElement>;

// Structs for a github tag
// https://api.github.com/repos/:owner/:repo/tags
#[derive(Debug, Serialize, Deserialize)]
pub struct GithubTagElement {
    pub(crate) name: String,
    commit: Commit,
    zipball_url: String,
    tarball_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    sha: String,
    url: String,
}

// This is the struct and implementation for a ShardManager Container,
// which allows for non serenity items to access the shardmanger,
use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::Mutex as SernMutex;
use sqlx::{
    PgConnection,
    Pool,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex,
    },
};
use tokio::runtime::Runtime;
use typemap::Key;

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<SernMutex<ShardManager>>;
}

pub struct SettingsContainer;

impl Key for SettingsContainer {
    type Value = Arc<Mutex<config::Config>>;
}

pub struct TokioContainer;

impl Key for TokioContainer {
    type Value = Arc<Mutex<Runtime>>;
}

pub struct PoolContainer;

impl Key for PoolContainer {
    type Value = Pool<PgConnection>;
}

pub struct PrefixHashMapContainer;

impl Key for PrefixHashMapContainer {
    type Value = Arc<Mutex<HashMap<u64, String>>>;
}
