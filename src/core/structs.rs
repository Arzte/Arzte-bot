use serde::{
    Deserialize,
    Serialize,
};

// Structs for the latest github release API response
// https://api.github.com/repos/:owner/:repo/releases/latest
#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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

// These are all structs for the github response to
// https://api.github.com/repos/:owner/:repo/commits/:commit
#[derive(Serialize, Deserialize)]
pub struct GithubCommit {
    pub(crate) sha: String,
    node_id: String,
    pub(crate) commit: Commit,
    url: String,
    html_url: String,
    comments_url: String,
    author: GithubCommitAuthor,
    committer: GithubCommitAuthor,
    parents: Vec<Parent>,
    stats: Stats,
    files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
struct GithubCommitAuthor {
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

#[derive(Serialize, Deserialize)]
pub struct Commit {
    author: CommitAuthor,
    committer: CommitAuthor,
    message: String,
    tree: Tree,
    url: String,
    comment_count: i64,
    verification: Verification,
}

#[derive(Serialize, Deserialize)]
struct CommitAuthor {
    name: String,
    email: String,
    date: String,
}

#[derive(Serialize, Deserialize)]
struct Tree {
    sha: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Verification {
    verified: bool,
    reason: String,
    signature: Option<serde_json::Value>,
    payload: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct File {
    sha: String,
    filename: String,
    status: String,
    additions: i64,
    deletions: i64,
    changes: i64,
    blob_url: String,
    raw_url: String,
    contents_url: String,
    patch: String,
}

#[derive(Serialize, Deserialize)]
struct Parent {
    sha: String,
    url: String,
    html_url: String,
}

#[derive(Serialize, Deserialize)]
struct Stats {
    total: i64,
    additions: i64,
    deletions: i64,
}

// This is the struct and implementation for a ShardManager Container,
// which allows for non serenity items to access the shardmanger,

use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::Mutex as SernMutex;
use std::sync::{
    Arc,
    Mutex,
};
use typemap::Key;

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<SernMutex<ShardManager>>;
}

pub struct SettingsContainer;

impl Key for SettingsContainer {
    type Value = Arc<Mutex<config::Config>>;
}
