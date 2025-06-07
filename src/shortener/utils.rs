use rand::{Rng as _, distr::Alphanumeric};

use crate::db::UrlDbSchema;

use super::{ShortenedUrl, Slug, Url};

pub fn parse_url(custom_url: String, url: Url, slug: Option<Slug>) -> UrlDbSchema {
    UrlDbSchema {
        shortened_url: ShortenedUrl {
            url: custom_url.into(),
            slug: generate_random_slug(),
        },
        parsed_url: url,
        parsed_slug: slug,
        clicks: 0,
    }
}

fn generate_random_slug() -> Slug {
    let mut rng = rand::rng();
    (0..7)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect::<String>()
        .into()
}
