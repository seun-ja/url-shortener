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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortened_slug: Option<Slug>,
}

pub struct EventManager {
    pub db: InMemDatabase,
}

impl EventManager {
    pub fn record_event(
        &mut self,
        action: Action,
        backup_file_path: &str,
        custom_url: &str,
    ) -> Result<ShortenedUrl, UrlShortenerError> {
        match action {
            Action::CreateShortLink(parsed_url, parsed_slug) => {
                let shortened_url = parse_url(
                    custom_url.to_string(),
                    parsed_url.clone(),
                    parsed_slug.clone(),
                );
                self.db.insert_url(shortened_url).inspect(|shortened_url| {
                    if let Err(err) = self.db.insert_event(
                        Action::CreateShortLink(parsed_url, parsed_slug),
                        backup_file_path,
                        Some(shortened_url.slug.clone()),
                    ) {
                        // TODO: ensure it's saved
                        tracing::error!("{err}");
                    };
                })
            }
            Action::Redirect(slug) => {
                self.db
                    .insert_event(Action::Redirect(slug.clone()), backup_file_path, None)?;
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
