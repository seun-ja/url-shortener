use crate::graphql::controller::{UrlMutations, UrlQueries};
use async_graphql::MergedObject;
use async_graphql::extensions::Tracing;
use async_graphql::{EmptySubscription, Schema};
use url_shortener::UrlShortener;

use std::sync::Mutex;

#[derive(MergedObject, Default)]
pub struct Query(UrlQueries);

#[derive(MergedObject, Default)]
pub struct Mutation(UrlMutations);

pub fn build_schema(backup_file_path: String) -> Schema<Query, Mutation, EmptySubscription> {
    let url_shortener = UrlShortener::init(backup_file_path);

    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Tracing)
        .data(Mutex::new(url_shortener))
        .finish()
}
