use crate::shortener::ShortenedUrl;
use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};

/// A record of the metrics recorded for each instance of [`ShortenedUrl`]
#[derive(Debug, Clone, PartialEq, SimpleObject, Serialize, Deserialize)]
pub struct Stats {
    pub link: ShortenedUrl,
    pub redirects: u64,
}
