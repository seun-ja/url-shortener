use serde::{Deserialize, Serialize};

use crate::{db::InMemDatabase, error::UrlShortenerError, shortener::utils::parse_url};

use super::{ShortenedUrl, Slug, Url};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    // Actions here for which to create events
    CreateShortLink(Url, Option<Slug>),
    Redirect(Slug),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub action: Action,
    pub timestamp: std::time::SystemTime,
}

#[derive(Default)]
pub struct EventManager {
    pub db: InMemDatabase,
}

impl EventManager {
    pub fn record_event(
        &mut self,
        action: Action,
        backup_file_path: &str,
    ) -> Result<ShortenedUrl, UrlShortenerError> {
        match action {
            Action::CreateShortLink(parsed_url, parsed_slug) => {
                let shortened_url = parse_url(parsed_url.clone(), parsed_slug.clone());
                self.db.insert_event(
                    Action::CreateShortLink(parsed_url, parsed_slug),
                    backup_file_path,
                )?;
                self.db.insert_url(shortened_url)
            }
            Action::Redirect(slug) => {
                self.db
                    .insert_event(Action::Redirect(slug.clone()), backup_file_path)?;
                self.db.get_url_by_slug(slug)
            }
        }
    }

    pub fn get_events(&self) -> Result<Vec<Event>, UrlShortenerError> {
        Ok(self.db.get_events())
    }

    // pub fn reload_from_backup(&self, data: Vec<Event>) -> Result<(), UrlShortenerError> {
    //     todo!()
    // }
}
