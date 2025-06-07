use std::{fmt::Display, fs::OpenOptions, io::Write as _};

use crate::db::BackUpData;
use crate::shortener::error::UrlShortenerError;
use crate::shortener::mutations::Mutations;
use crate::shortener::queries::Query;
use crate::shortener::stats::Stats;
use crate::{db::InMemDatabase, event::EventManager};
use async_graphql::{NewType, SimpleObject};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod event;
pub mod mutations;
pub mod queries;
pub mod stats;
pub mod utils;

/// A newtype representing the URLs that are provided by the user to generate short links for.
#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, NewType, Serialize, Deserialize)]
pub struct Url(String);

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A newtype representing the short link slug for URLs provided by the user.
#[derive(
    Debug, Default, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, NewType, Serialize, Deserialize,
)]
pub struct Slug(String);

/// A record of the complete URL and its short link slug.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, SimpleObject, Serialize, Deserialize)]
pub struct ShortenedUrl {
    pub url: Url,
    pub slug: Slug,
}

/// Struct representing the URL shortening service.
pub struct UrlShortener {
    event_manager: EventManager,
    pub backup_file_path: String,
    pub custom_url: String,
}

impl UrlShortener {
    pub fn init(backup_file_path: String, custom_url: String, backup_data: BackUpData) -> Self {
        Self {
            event_manager: EventManager {
                db: InMemDatabase::init(backup_data),
            },
            backup_file_path,
            custom_url,
        }
    }
}

pub fn update_backup_data(content: &str, file_path: &str) -> Result<(), UrlShortenerError> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?;

    // Write the content to the file
    file.write_all(content.as_bytes())
        .map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))
        .map(|_| ())
}

impl Query for UrlShortener {
    /// For a given URL slug, return the current stats, or an error if the slug doesn't exist.
    fn get_metrics(&self, slug: Slug) -> Result<Stats, UrlShortenerError> {
        self.event_manager.db.slug_clicks_stats(slug)
    }
}

impl Mutations for UrlShortener {
    /// Create a new short link for the provided URL. The URL is also validated before creating a slug
    /// to ensure only valid URLs are accepted.
    ///
    /// # Arguments
    /// * `url`: The URL to generate short link for
    /// * `slug`: Optional slug parameter to use for the URL. If the provided slug is already in use,
    ///   this mutation should return an error.
    fn create_short_link(
        &mut self,
        url: Url,
        slug: Option<Slug>,
    ) -> Result<ShortenedUrl, UrlShortenerError> {
        self.event_manager.record_event(
            event::Action::CreateShortLink(url, slug),
            &self.backup_file_path,
            &self.custom_url,
        )
    }

    /// For a given slug, look-up the original URL and return that, or return an error if the slug is
    /// invalid. Also, the `hits` metric should be increased by 1 each time redirect is called for a slug.
    ///
    /// In addition, an event needs to be recorded that indicates this action, and the data associated with it.
    ///
    /// # Arguments
    /// * `slug`: Short link slug to find the complete URL for.
    fn redirect(&mut self, slug: Slug) -> Result<ShortenedUrl, UrlShortenerError> {
        self.event_manager.record_event(
            event::Action::Redirect(slug),
            &self.backup_file_path,
            &self.custom_url,
        )
    }

    /// Reload the state of this service from a backup file. This must be executed when the service
    /// is initialised in order to ensure that we can recover from restarts.
    ///
    /// In addition, an event needs to be recorded to indicate that the service was recovered from a
    /// backup.
    fn reload_from_backup(&mut self, backup_file_path: &str) -> Result<(), UrlShortenerError> {
        self.event_manager
            .db
            .reload_db_from_backup(backup_file_path)
    }
}
