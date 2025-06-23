use std::{
    collections::{HashMap, hash_map},
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, RwLock},
    time::SystemTime,
};

use crate::{
    Config, ParsedSlug, ParsedUrl, ShortenedUrl, Slug,
    error::UrlShortenerError,
    event::{Action, Event},
    stats::Stats,
    update_backup_data,
    utils::parse_url,
};

pub type BackUpData = (HashMap<Slug, UrlDbSchema>, Vec<Event>);

#[derive(Clone, Default)]
/// In-memory database for shortened url management
pub struct InMemDatabase {
    url_db: Arc<RwLock<HashMap<Slug, UrlDbSchema>>>,
    events_db: Arc<RwLock<Vec<Event>>>,
}

impl InMemDatabase {
    pub fn init(backedup_data: BackUpData) -> Self {
        Self {
            url_db: Arc::new(RwLock::new(backedup_data.0)),
            events_db: Arc::new(RwLock::new(backedup_data.1)),
        }
    }
}

pub fn load_from_backup(config: &Config) -> Result<BackUpData, Box<dyn std::error::Error>> {
    let mut new_event_store = Vec::new();
    let mut new_url_store = HashMap::new();
    if let Ok(file) = File::open(&config.backup_file_path) {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let event: Event = serde_json::from_str(&line?)?;
            new_event_store.push(event.clone());
            match event.action {
                Action::CreateShortLink(url, slug) => {
                    new_url_store.insert(
                        slug.clone().unwrap_or("".to_string().into()),
                        UrlDbSchema {
                            shortened_url: ShortenedUrl {
                                url: config.custom_url.clone().into(),
                                slug: event.shortened_slug.clone().unwrap_or_default(),
                            },
                            parsed_url: url,
                            parsed_slug: slug,
                            clicks: 0, // TODO: Get real value
                        },
                    );
                }
                Action::Redirect(_) => (),
            }
        }

        tracing::info!(
            "Database backup loaded with {} events data & {} urls",
            new_event_store.len(),
            new_url_store.len()
        );
    } else {
        File::create(&config.backup_file_path)?;
    };

    Ok((new_url_store, new_event_store))
}

#[derive(Clone, Debug)]
pub struct UrlDbSchema {
    pub shortened_url: ShortenedUrl, // TODO: unique to user/ip
    pub parsed_url: ParsedUrl,
    pub parsed_slug: Option<ParsedSlug>,
    pub clicks: u64,
}

impl InMemDatabase {
    /// Stores a `ShortenedUrl` to database
    pub fn insert_url(
        &self,
        custom_url: &str,
        parsed_url: ParsedUrl,
        parsed_slug: Option<ParsedSlug>,
    ) -> Result<ShortenedUrl, UrlShortenerError> {
        let shortened_url = parse_url(
            custom_url.to_string(),
            parsed_url.clone(),
            parsed_slug.clone(),
        );
        match self
            .url_db
            .write()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .entry(shortened_url.shortened_url.slug.clone())
        {
            hash_map::Entry::Vacant(e) => {
                e.insert(shortened_url.clone());
                Ok(shortened_url.shortened_url)
            }
            hash_map::Entry::Occupied(url) => {
                tracing::warn!("Url already exists");
                Ok(url.into_mut().shortened_url.clone())
            }
        }
    }

    /// Get a `ShortenedUrl` from database
    pub fn get_url_by_slug(&self, slug: Slug) -> Result<ShortenedUrl, UrlShortenerError> {
        let url = self
            .url_db
            .read()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .get(&slug)
            .ok_or(UrlShortenerError::NotFound)
            .map(|record| ShortenedUrl {
                url: record.parsed_url.clone(),
                slug: record.parsed_slug.clone().unwrap_or("".to_string().into()),
            })?;

        if let Some(e) = self
            .url_db
            .write()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .get_mut(&url.slug)
        {
            e.clicks += 1;
            tracing::info!("clicked {} time(s)", e.clicks);
        };

        Ok(url)
    }

    pub fn slug_clicks_stats(&self, slug: Slug) -> Result<Stats, UrlShortenerError> {
        self.url_db
            .read()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .get(&slug)
            .ok_or(UrlShortenerError::NotFound)
            .map(|record| Stats {
                link: record.shortened_url.clone(),
                redirects: record.clicks,
            })
    }

    pub fn insert_event(
        &self,
        action: Action,
        backup_file_path: &str,
        shortened_slug: Option<Slug>,
    ) -> Result<(), UrlShortenerError> {
        let timestamp = SystemTime::now();
        let event = Event {
            action,
            timestamp,
            shortened_slug,
        };
        update_backup_data(
            &format!(
                "{}\n",
                serde_json::to_string(&event)
                    .map_err(|err| UrlShortenerError::SerializationError(err.to_string()))?
            ),
            backup_file_path,
        )?;
        self.events_db
            .write()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .push(event);

        Ok(())
    }

    pub fn reload_db_from_backup(&self, backup_file_path: &str) -> Result<(), UrlShortenerError> {
        let file = File::open(backup_file_path)
            .map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let event: Event = serde_json::from_str(
                &line.map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?,
            )
            .map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?;
            self.events_db
                .write()
                .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
                .push(event)
        }

        Ok(())
    }

    pub fn get_events(&self) -> Result<Vec<Event>, UrlShortenerError> {
        Ok(self
            .events_db
            .read()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .clone())
    }

    pub fn get_urls(&self) -> Result<HashMap<Slug, UrlDbSchema>, UrlShortenerError> {
        Ok(self
            .url_db
            .read()
            .map_err(|e| UrlShortenerError::DatabaseError(e.to_string()))?
            .clone())
    }
}
