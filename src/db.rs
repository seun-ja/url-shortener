use std::{
    collections::{BTreeMap, btree_map},
    fs::File,
    io::{BufRead, BufReader},
    time::SystemTime,
};

use crate::{
    Config, ShortenedUrl, Slug, Url,
    error::UrlShortenerError,
    event::{Action, Event},
    stats::Stats,
    update_backup_data,
};

pub type BackUpData = (BTreeMap<String, UrlDbSchema>, Vec<Event>);

#[derive(Clone, Default)]
/// In-memory database for shortened url management
pub struct InMemDatabase {
    url_db: BTreeMap<String, UrlDbSchema>,
    events_db: Vec<Event>,
}

impl InMemDatabase {
    pub fn init(backedup_data: BackUpData) -> Self {
        Self {
            url_db: backedup_data.0,
            events_db: backedup_data.1,
        }
    }
}

pub fn load_from_backup(config: &Config) -> Result<BackUpData, Box<dyn std::error::Error>> {
    let mut new_event_store = Vec::new();
    let mut new_url_store = BTreeMap::new();
    if let Ok(file) = File::open(&config.backup_file_path) {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let event: Event = serde_json::from_str(&line?)?;
            new_event_store.push(event.clone());
            match event.action {
                Action::CreateShortLink(url, slug) => {
                    new_url_store.insert(
                        url.to_string(),
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
    pub parsed_url: Url,
    pub parsed_slug: Option<Slug>,
    pub clicks: u64,
}

impl InMemDatabase {
    /// Stores a `ShortenedUrl` to database
    pub fn insert_url(&mut self, new_url: UrlDbSchema) -> Result<ShortenedUrl, UrlShortenerError> {
        if let btree_map::Entry::Vacant(e) = self.url_db.entry(new_url.parsed_url.to_string()) {
            e.insert(new_url.clone());
            Ok(new_url.shortened_url)
        } else {
            tracing::warn!("Url already exists");
            Err(UrlShortenerError::SlugAlreadyExists)
        }
    }

    /// Get a `ShortenedUrl` from database
    pub fn get_url_by_slug(&mut self, slug: Slug) -> Result<ShortenedUrl, UrlShortenerError> {
        let url = self
            .url_db
            .values()
            .find(|url| url.shortened_url.slug == slug)
            .ok_or(UrlShortenerError::NotFound)
            .map(|record| ShortenedUrl {
                url: record.parsed_url.clone(),
                slug: record.parsed_slug.clone().unwrap_or("".to_string().into()),
            })?;

        if let Some(e) = self.url_db.get_mut(&url.url.to_string()) {
            e.clicks += 1;
            tracing::info!("clicked {} time(s)", e.clicks);
        };

        Ok(url)
    }

    pub fn slug_clicks_stats(&self, slug: Slug) -> Result<Stats, UrlShortenerError> {
        self.url_db
            .values()
            .find(|url| url.shortened_url.slug == slug)
            .ok_or(UrlShortenerError::NotFound)
            .map(|record| Stats {
                link: record.shortened_url.clone(),
                redirects: record.clicks,
            })
    }

    pub fn insert_event(
        &mut self,
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
        self.events_db.push(event);

        Ok(())
    }

    pub fn reload_db_from_backup(
        &mut self,
        backup_file_path: &str,
    ) -> Result<(), UrlShortenerError> {
        let file = File::open(backup_file_path)
            .map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let event: Event = serde_json::from_str(
                &line.map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?,
            )
            .map_err(|e| UrlShortenerError::FileOpenError(e.to_string()))?;
            self.events_db.push(event)
        }

        Ok(())
    }

    pub fn get_events(&self) -> Vec<Event> {
        self.events_db.clone()
    }

    pub fn get_urls(&self) -> BTreeMap<String, UrlDbSchema> {
        self.url_db.clone()
    }
}
