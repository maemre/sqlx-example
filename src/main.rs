use std::collections::HashMap;

use clap::{Parser, Subcommand};
use sqlx::Connection;
use sqlx::sqlite::SqliteConnection;

#[derive(Parser)]
#[command(name = "bookmarks", about = "A simple bookmark manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Add a new bookmark
    Add {
        /// URL of the bookmark
        #[arg(long)]
        url: String,
        /// Title of the bookmark
        #[arg(long)]
        title: String,
        /// Comma-separated list of tags
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// List all bookmarks
    List,
    /// List bookmarks that have a given tag
    ListByTag {
        /// Tag name to filter by
        tag: String,
    },
}

async fn create_bookmark(
    conn: &mut SqliteConnection,
    url: &str,
    title: &str,
    tags: &[String],
) -> Result<(), sqlx::Error> {
    let bookmark_id = sqlx::query_scalar::<_, i64>(
        r"INSERT INTO bookmark (url, title) VALUES (?, ?) RETURNING id",
    )
    .bind(url)
    .bind(title)
    .fetch_one(&mut *conn)
    .await?;

    if !tags.is_empty() {
        let values = vec!["(?)"; tags.len()].join(", ");
        let insert_tags = format!(r"INSERT OR IGNORE INTO tag (name) VALUES {values}");
        let mut q = sqlx::query(&insert_tags);
        for tag in tags {
            q = q.bind(tag);
        }
        q.execute(&mut *conn).await?;

        let placeholders = vec!["?"; tags.len()].join(", ");
        let link_tags = format!(
            r"INSERT INTO bookmark_tag (bookmark_id, tag_id)
              SELECT ?, id FROM tag WHERE name IN ({placeholders})"
        );
        let mut q = sqlx::query(&link_tags).bind(bookmark_id);
        for tag in tags {
            q = q.bind(tag);
        }
        q.execute(&mut *conn).await?;
    }

    println!("Added bookmark: {title} ({url})");
    Ok(())
}

async fn list_bookmarks(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    let bookmarks = sqlx::query_as::<_, (i64, String, String)>(
        r"SELECT id, url, title FROM bookmark ORDER BY id",
    )
    .fetch_all(&mut *conn)
    .await?;

    if bookmarks.is_empty() {
        println!("No bookmarks found.");
        return Ok(());
    }

    let links = sqlx::query_as::<_, (i64, i64)>(
        r"SELECT bookmark_id, tag_id FROM bookmark_tag",
    )
    .fetch_all(&mut *conn)
    .await?;

    let tag_names: HashMap<i64, String> = sqlx::query_as::<_, (i64, String)>(
        r"SELECT id, name FROM tag WHERE id IN (SELECT tag_id FROM bookmark_tag)",
    )
    .fetch_all(&mut *conn)
    .await?
    .into_iter()
    .collect();

    let mut tags_by_bookmark: HashMap<i64, Vec<&str>> = HashMap::new();
    for (bookmark_id, tag_id) in &links {
        if let Some(name) = tag_names.get(tag_id) {
            tags_by_bookmark
                .entry(*bookmark_id)
                .or_default()
                .push(name.as_str());
        }
    }

    for (id, url, title) in &bookmarks {
        let mut tags = tags_by_bookmark.remove(id).unwrap_or_default();
        tags.sort();
        let tags_display = if tags.is_empty() {
            String::from("(none)")
        } else {
            tags.join(", ")
        };
        println!("[{id}] {title}");
        println!("    URL:  {url}");
        println!("    Tags: {tags_display}");
        println!();
    }
    Ok(())
}

async fn list_bookmarks_by_tag(conn: &mut SqliteConnection, tag: &str) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        r"SELECT b.id, b.url, b.title FROM bookmark b, bookmark_tag bt, tag t
          WHERE b.id = bt.bookmark_id AND t.id = bt.tag_id AND t.name = ?
          ORDER BY b.id",
    )
    .bind(tag)
    .fetch_all(&mut *conn)
    .await?;

    if rows.is_empty() {
        println!("No bookmarks found with tag '{tag}'.");
        return Ok(());
    }

    println!("Bookmarks tagged '{tag}':");
    for (id, url, title) in &rows {
        println!("  [{id}] {title}");
        println!("       {url}");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let cli = Cli::parse();

    let mut conn = SqliteConnection::connect("sqlite:bookmarks.db?mode=rwc").await?;

    sqlx::raw_sql(include_str!("../schema.sql"))
        .execute(&mut conn)
        .await?;

    match cli.command {
        Command::Add { url, title, tags } => {
            create_bookmark(&mut conn, &url, &title, &tags).await?;
        }
        Command::List => {
            list_bookmarks(&mut conn).await?;
        }
        Command::ListByTag { tag } => {
            list_bookmarks_by_tag(&mut conn, &tag).await?;
        }
    }

    conn.close().await?;
    Ok(())
}
