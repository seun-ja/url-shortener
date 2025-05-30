use crate::graphql::GraphQLService;
use crate::graphql::schema::build_schema;
use axum::Router;
use axum::serve::Serve;
use tokio::net::TcpListener;
use url_shortener::db::load_from_backup;

pub async fn start(
    backup_file_path: String,
) -> std::io::Result<Serve<TcpListener, Router, Router>> {
    let backup = load_from_backup(&backup_file_path).expect("Unable to start service");
    let schema = build_schema(backup_file_path);
    let service = GraphQLService::new("url", "8080", schema, backup);

    service.start().await
}
