use std::sync::Mutex;

use async_graphql::{Context, Error, Object, Result};
use url_shortener::mutations::Mutations;
use url_shortener::queries::Query;
use url_shortener::stats::Stats;
use url_shortener::{ShortenedUrl, Slug, Url, UrlShortener};

#[derive(Default)]
pub struct UrlQueries;

#[Object]
impl UrlQueries {
    pub async fn stats(&self, ctx: &Context<'_>, slug: Slug) -> Result<Stats> {
        let url_shortener = ctx
            .data_unchecked::<Mutex<UrlShortener>>()
            .lock()
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(url_shortener.get_metrics(slug)?)
    }
}

#[derive(Default)]
pub struct UrlMutations;

#[Object]
impl UrlMutations {
    pub async fn create_short_link(
        &self,
        ctx: &Context<'_>,
        link: Url,
        slug: Option<Slug>,
    ) -> Result<ShortenedUrl> {
        let mut url_shortener = ctx
            .data_unchecked::<Mutex<UrlShortener>>()
            .lock()
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(url_shortener.create_short_link(link, slug)?)
    }

    pub async fn redirect(&self, ctx: &Context<'_>, slug: Slug) -> Result<ShortenedUrl> {
        let mut url_shortener = ctx
            .data_unchecked::<Mutex<UrlShortener>>()
            .lock()
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(url_shortener.redirect(slug)?)
    }
}
