# URL Shortener

A simple URL shortener service built with Rust. This project allows users to shorten long URLs and redirect to the original URLs using the generated short links.

## Features

- Shorten long URLs to easy-to-share short links
- Redirect short URLs to the original destination
- GraphQL API
- Simple and easy to deploy
- Backup event record

## Getting Started

### Installation

```bash
git clone https://github.com/seun-ja/url-shortener.git
cd url-shortener
```

### Running the Application

```bash
cargo run
```

The server will start on `http://localhost:8080`.

### GraphQL Playground

```bash
http://127.0.0.1:8080/url
```

## Mutations and Query

### Mutation

```graphql
createShortLink(
    link: String!
    slug: String
): ShortenedUrl!
```

```graphql
redirect(
    slug: String!
): ShortenedUrl!
```

### Query

```graphql
stats(
    slug: String!
): Stats!
```

## Schema

Schema file available at `./schema.graphql

## Configuration

You can configure the backup file location in `.env`. Provide `BACKUP_PATH_FILE`.
