use crate::graphql::controller::{UrlMutations, UrlQueries};
use async_graphql::MergedObject;
use async_graphql::extensions::Tracing;
use async_graphql::{EmptySubscription, Schema};
use std::sync::Mutex;
use url_shortener::UrlShortener;
use url_shortener::db::BackUpData;

#[derive(MergedObject, Default)]
pub struct Query(UrlQueries);

#[derive(MergedObject, Default)]
pub struct Mutation(UrlMutations);

pub fn build_schema(
    backup_file_path: String,
    custom_url: String,
    backup_data: BackUpData,
) -> Schema<Query, Mutation, EmptySubscription> {
    let url_shortener = UrlShortener::init(backup_file_path, custom_url, backup_data);

    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .extension(Tracing)
        .data(Mutex::new(url_shortener))
        .finish()
}
