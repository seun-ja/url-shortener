use crate::graphql::GraphQLService;
use crate::graphql::schema::build_schema;
use axum::Router;
use axum::serve::Serve;
use tokio::net::TcpListener;
use url_shortener::Config;
use url_shortener::db::load_from_backup;

pub async fn start(config: Config) -> std::io::Result<Serve<TcpListener, Router, Router>> {
    let backedup_data = load_from_backup(&config).expect("Unable to start service");
    let schema = build_schema(config.backup_file_path, config.custom_url, backedup_data);
    let service = GraphQLService::new("url", "8080", schema);

    service.start().await
}
