use crate::shortener::Slug;
use crate::shortener::error::UrlShortenerError;
use crate::shortener::stats::Stats;

pub trait Query {
    /// For a given URL slug, return the current stats, or an error if the slug doesn't exist.
    fn get_metrics(&self, slug: Slug) -> Result<Stats, UrlShortenerError>;
}
