use crate::shortener::error::UrlShortenerError;
use crate::shortener::{ShortenedUrl, Slug, Url};

pub trait Mutations {
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
    ) -> Result<ShortenedUrl, UrlShortenerError>;

    /// For a given slug, look-up the original URL and return that, or return an error if the slug is
    /// invalid. Also, the `hits` metric should be increased by 1 each time redirect is called for a slug.
    ///
    /// In addition, an event needs to be recorded that indicates this action, and the data associated with it.
    ///
    /// # Arguments
    /// * `slug`: Short link slug to find the complete URL for.
    fn redirect(&mut self, slug: Slug) -> Result<ShortenedUrl, UrlShortenerError>;

    /// Reload the state of this service from a backup file. This must be executed when the service
    /// is initialised in order to ensure that we can recover from restarts.
    ///
    /// In addition, an event needs to be recorded to indicate that the service was recovered from a
    /// backup.
    fn reload_from_backup(&mut self, backup_file_path: &str) -> Result<(), UrlShortenerError>;
}
