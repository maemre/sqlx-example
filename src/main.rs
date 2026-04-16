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
        "INSERT INTO bookmark (url, title) VALUES (?, ?) RETURNING id",
    )
    .bind(url)
    .bind(title)
    .fetch_one(&mut *conn)
    .await?;

    for tag in tags {
        sqlx::query("INSERT OR IGNORE INTO tag (name) VALUES (?)")
            .bind(tag)
            .execute(&mut *conn)
            .await?;

        let tag_id =
            sqlx::query_scalar::<_, i64>("SELECT id FROM tag WHERE name = ?")
                .bind(tag)
                .fetch_one(&mut *conn)
                .await?;

        sqlx::query("INSERT INTO bookmark_tag (bookmark_id, tag_id) VALUES (?, ?)")
            .bind(bookmark_id)
            .bind(tag_id)
            .execute(&mut *conn)
            .await?;
    }

    println!("Added bookmark: {title} ({url})");
    Ok(())
}

async fn list_bookmarks(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, url, title FROM bookmark ORDER BY id",
    )
    .fetch_all(&mut *conn)
    .await?;

    if rows.is_empty() {
        println!("No bookmarks found.");
        return Ok(());
    }

    for (id, url, title) in &rows {
        let tag_rows = sqlx::query_as::<_, (String,)>(
            "SELECT t.name FROM tag t \
             JOIN bookmark_tag bt ON t.id = bt.tag_id \
             WHERE bt.bookmark_id = ? \
             ORDER BY t.name",
        )
        .bind(id)
        .fetch_all(&mut *conn)
        .await?;

        let tags: Vec<&str> = tag_rows.iter().map(|(name,)| name.as_str()).collect();
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

async fn list_bookmarks_by_tag(
    conn: &mut SqliteConnection,
    tag: &str,
) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT b.id, b.url, b.title FROM bookmark b \
         JOIN bookmark_tag bt ON b.id = bt.bookmark_id \
         JOIN tag t ON t.id = bt.tag_id \
         WHERE t.name = ? \
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
