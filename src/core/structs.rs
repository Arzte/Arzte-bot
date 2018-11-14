// These are all structs for the github response to
// https://api.github.com/repos/:owner/:repo/commits/:commit
#[derive(Serialize, Deserialize)]
pub struct Github {
    pub sha: String,
    node_id: String,
    commit: Commit,
    url: String,
    html_url: String,
    comments_url: String,
    author: GithubAuthor,
    committer: GithubAuthor,
    parents: Vec<Parent>,
    stats: Stats,
    files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
pub struct GithubAuthor {
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
pub struct CommitAuthor {
    name: String,
    email: String,
    date: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tree {
    sha: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Verification {
    verified: bool,
    reason: String,
    signature: Option<serde_json::Value>,
    payload: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
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
pub struct Parent {
    sha: String,
    url: String,
    html_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Stats {
    total: i64,
    additions: i64,
    deletions: i64,
}

// This is the struct and implementation for a ShardManager Container,
// which allows for non serenity items to access the shardmanger,
// after locking it.

use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::Mutex;
use std::sync::Arc;
use typemap::Key;

pub struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}