use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use sqlx::AnyPool;
use sqlx::any::AnyPoolOptions;
use futures::stream::{StreamExt, TryStreamExt};
use thiserror::Error;

/// Concurrency used for batched DB writes (hides per-statement round-trip latency).
const DB_CONCURRENCY: usize = 16;

use pcss;
use pcss::api_types::UiTexts;
use crate::models::{Document, DocumentLinks, DocumentText, Illustration, WorkshopImage, ToolImage, MediaImage, Part, TreeNode, TreeNodeLinks, Vehicle, Translations, Tool, ToolDistributor, ToolDistributorsLink, ReferencingToolDocument};
use crate::settings;
use crate::settings::Settings;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid tree node id {0}")]
    NoSuchTreeNodeId(i32),

    #[error("invalid tree node location {0}")]
    NoSuchTreeNode(String),

    #[error("invalid illustration {0}")]
    NoSuchIllustration(i32),

    #[error("invalid document {0}")]
    NoSuchDocument(String),

    #[error("invalid tool {0}")]
    NoSuchToolImage(String),

    #[error("invalid tool {0}")]
    NoSuchTool(String),

    #[error("invalid media {0}")]
    NoSuchMedia(String),

    #[error("invalid image {0}")]
    NoSuchImage(i32),

    #[error("no such vehicle type {1} in year {0}")]
    NoSuchVehicle(i32, String),

    #[error("error getting translations")]
    ErrorGettingTranslations,
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(self,
            Error::NoSuchTreeNodeId(_) | Error::NoSuchTreeNode(_) |
            Error::NoSuchIllustration(_) | Error::NoSuchDocument(_) |
            Error::NoSuchToolImage(_) | Error::NoSuchTool(_) |
            Error::NoSuchMedia(_) | Error::NoSuchImage(_) |
            Error::NoSuchVehicle(_, _)
        )
    }
}

pub struct ContentStore {
    pool: AnyPool,
    vehicle_image_path: String,
    is_postgres: bool,
}

impl PartialEq<TreeNode> for TreeNode {
    fn eq(&self, other: &TreeNode) -> bool {
        self.vehicle == other.vehicle &&
            self.year == other.year &&
            self.node_value == other.node_value &&
            self.name == other.name &&
            self.illustration_id == other.illustration_id &&
            self.location == other.location
    }
}

impl PartialEq<Document> for &Document {
    fn eq(&self, other: &Document) -> bool {
        self.hkap_id == other.hkap_id &&
            self.variant_id == other.variant_id &&
            self.language_code == other.language_code &&
            self.version == other.version &&
            self.vehicle_component == other.vehicle_component &&
            self.title == other.title &&
            self.document_type == other.document_type &&
            self.publication_date == other.publication_date &&
            self.file_format == other.file_format &&
            self.vehicle_component_with_document_index == other.vehicle_component_with_document_index &&
            self.new == other.new &&
            self.bookmarked == other.bookmarked &&
            self.content == other.content
    }
}

impl PartialEq<Part> for &Part {
    fn eq(&self, other: &Part) -> bool {
        self.vehicle == other.vehicle &&
            self.year == other.year &&
            self.part_id == other.part_id &&
            self.paw_relevant == other.paw_relevant &&
            self.text == other.text &&
            self.vehicle_component == other.vehicle_component
    }
}

impl PartialEq<TreeNodeLinks> for &TreeNodeLinks {
    fn eq(&self, other: &TreeNodeLinks) -> bool {
        self.child_node_id == other.child_node_id && self.parent_node_id == other.parent_node_id
    }
}

pub fn squish(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if !ch.is_whitespace() {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

/// Handle to the Tokio runtime, captured once at startup. Lets `block` drive futures from
/// non-runtime threads (the rayon import workers) — see `block`.
static RT_HANDLE: std::sync::OnceLock<tokio::runtime::Handle> = std::sync::OnceLock::new();

/// Run an async future to completion while blocking the current thread. Works from three
/// contexts:
/// - a Tokio worker (API handlers, the import driver): `block_in_place` yields to the runtime;
/// - a plain rayon worker (the import fetch/write threads): drive it on this thread via the
///   captured handle. Keeping the DB write on the same thread that allocated and will free the
///   image bytes is what avoids the cross-thread-free allocator thrash that made the parallel
///   loader pathologically slow.
fn block<F, T>(fut: F) -> Result<T>
where
    F: std::future::Future<Output = sqlx::Result<T>>,
{
    Ok(match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(fut)),
        Err(_) => RT_HANDLE
            .get()
            .expect("RT_HANDLE not initialized (construct a ContentStore first)")
            .block_on(fut),
    }?)
}

/// Rewrite `?` placeholders to `$1, $2, ...` for Postgres.
/// AnyPool does not do this automatically.
fn pg(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len() + 16);
    let mut n = 0usize;
    for c in sql.chars() {
        if c == '?' {
            n += 1;
            out.push('$');
            out.push_str(&n.to_string());
        } else {
            out.push(c);
        }
    }
    out
}

impl ContentStore {
    /// Rewrite `?` placeholders to `$N` for Postgres (AnyPool doesn't do this automatically).
    fn q(&self, query: &str) -> String {
        if self.is_postgres { pg(query) } else { query.to_string() }
    }

    pub fn new(settings: &Settings) -> Self {
        sqlx::any::install_default_drivers();
        // Capture the runtime handle (we're on a Tokio thread here) so `block` can drive DB
        // writes from the rayon import workers later.
        let _ = RT_HANDLE.get_or_init(|| tokio::runtime::Handle::current());
        let url = &settings.database_url;
        let is_postgres = url.starts_with("postgres");
        // The content DB is a re-loadable cache of PCSS/source data, so per-commit
        // durability isn't needed. Relax fsync-per-commit on every connection: this
        // is the dominant cost of the bulk loaders (thousands of tiny autocommit
        // INSERTs, each otherwise fsync'd). Postgres → async commit; SQLite → WAL.
        let opts = AnyPoolOptions::new().max_connections((DB_CONCURRENCY + 2) as u32);
        let opts = if is_postgres {
            opts.after_connect(|conn, _meta| Box::pin(async move {
                sqlx::query("SET synchronous_commit = off").execute(conn).await?;
                Ok(())
            }))
        } else {
            opts.after_connect(|conn, _meta| Box::pin(async move {
                sqlx::query("PRAGMA journal_mode = WAL").execute(&mut *conn).await?;
                sqlx::query("PRAGMA synchronous = NORMAL").execute(conn).await?;
                Ok(())
            }))
        };
        let pool = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(opts.connect(url))
        }).expect("Failed to connect to database");
        ContentStore {
            pool,
            vehicle_image_path: settings.importer.vehicle_image_path.clone(),
            is_postgres,
        }
    }

    pub fn run_migrations(&self, _settings: &Settings) -> Result<()> {
        let migration_dir = if self.is_postgres {
            "migrations/postgres"
        } else {
            "migrations/sqlite"
        };
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::migrate::Migrator::new(Path::new(migration_dir))
                    .await?
                    .run(&self.pool)
                    .await
            })
        })?;
        Ok(())
    }

    pub fn get_parent(&self, id: i32) -> Result<i32> {
        let results: Vec<TreeNodeLinks> = block(
            sqlx::query_as::<_, TreeNodeLinks>(
                "SELECT id, parent_node_id, child_node_id FROM tree_node_links WHERE child_node_id = ?"
            )
            .bind(id)
            .fetch_all(&self.pool)
        )?;

        if results.is_empty() {
            bail!("id is invalid")
        }
        Ok(results[0].parent_node_id)
    }

    pub fn get_links(&self, parent_id: i32) -> Result<Vec<i32>> {
        let q = self.q("SELECT child_node_id FROM tree_node_links WHERE parent_node_id = ?");
        let results: Vec<(i32,)> = block(
            sqlx::query_as::<_, (i32,)>(&q)
            .bind(parent_id)
            .fetch_all(&self.pool)
        )?;
        Ok(results.into_iter().map(|r| r.0).collect())
    }

    pub fn search_documents(&self, vehicle: &String, year: i32, document_type: &String, apos_number: &String) -> Result<Vec<Document>> {
        println!("search_documents: vehicle: {}, year: {}, document_type: {}, apos_number: {}", vehicle, year, document_type, apos_number);
        let q = self.q("SELECT DISTINCT d.hkap_id, d.variant_id, d.language_code, d.version, d.vehicle_component, \
                 d.title, d.document_type, d.publication_date, d.file_format, \
                 d.vehicle_component_with_document_index, d.new, d.bookmarked, d.content \
                 FROM documents d \
                 JOIN document_links dl ON dl.hkap_id = d.hkap_id \
                 JOIN tree_nodes tn ON tn.node_id = dl.node_id \
                 WHERE tn.vehicle = ? AND tn.year = ? \
                 AND d.vehicle_component_with_document_index = ?");
        let results: Vec<Document> = block(
            sqlx::query_as::<_, Document>(&q)
            .bind(vehicle)
            .bind(year)
            .bind(apos_number)
            .fetch_all(&self.pool)
        )?;
        Ok(results)
    }

    pub fn list_documents_by_node_id(&self, node_id: i32) -> Result<Vec<Document>> {
        let node = self.get_tree_node_by_id(node_id)?;
        if node.location == Some("000".into()) {
            return Ok(vec![]);
        }

        let q = self.q("WITH RECURSIVE sub(id) AS (\
                   SELECT node_id FROM tree_nodes WHERE node_id = ? \
                   UNION ALL \
                   SELECT tnl.child_node_id FROM tree_node_links tnl JOIN sub ON sub.id = tnl.parent_node_id\
                 ) \
                 SELECT DISTINCT d.hkap_id, d.variant_id, d.language_code, d.version, d.vehicle_component, \
                 d.title, d.document_type, d.publication_date, d.file_format, \
                 d.vehicle_component_with_document_index, d.new, d.bookmarked, d.content \
                 FROM documents d \
                 JOIN document_links dl ON dl.hkap_id = d.hkap_id \
                 WHERE dl.node_id IN (SELECT id FROM sub)");
        let mut results: Vec<Document> = block(
            sqlx::query_as::<_, Document>(&q)
            .bind(node_id)
            .fetch_all(&self.pool)
        )?;
        results.sort();
        Ok(results)
    }

    pub fn search_documents_by_vehicle(&self, vehicle: &str, year: i32, query: &str) -> Result<Vec<Document>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(vec![]);
        }

        let like = format!("%{}%", escape_like(query));
        let like_squished = format!("%{}%", escape_like(&squish(query)));
        let cast_content = if self.is_postgres { "convert_from(d.content, 'UTF8')" } else { "CAST(d.content AS TEXT)" };
        let sql = format!(
            "SELECT DISTINCT d.hkap_id, d.variant_id, d.language_code, d.version, d.vehicle_component, \
             d.title, d.document_type, d.publication_date, d.file_format, \
             d.vehicle_component_with_document_index, d.new, d.bookmarked, d.content \
             FROM documents d \
             JOIN document_links dl ON dl.hkap_id = d.hkap_id \
             JOIN tree_nodes tn ON tn.node_id = dl.node_id \
             WHERE tn.vehicle = ? AND tn.year = ? \
             AND (\
               d.title LIKE ? ESCAPE '\\' \
               OR d.vehicle_component_with_document_index LIKE ? ESCAPE '\\' \
               OR (d.file_format = 'xml' AND {cast_content} LIKE ? ESCAPE '\\') \
               OR (d.file_format = 'pdf' AND EXISTS (\
                 SELECT 1 FROM document_text dt \
                 WHERE dt.hkap_id = d.hkap_id \
                 AND (dt.text LIKE ? ESCAPE '\\' OR dt.text_squished LIKE ? ESCAPE '\\')\
               ))\
             )"
        );

        let sql = self.q(&sql);
        let mut results: Vec<Document> = block(
            sqlx::query_as::<_, Document>(&sql)
                .bind(vehicle)
                .bind(year)
                .bind(&like)
                .bind(&like)
                .bind(&like)
                .bind(&like)
                .bind(&like_squished)
                .fetch_all(&self.pool)
        )?;
        results.sort();
        Ok(results)
    }

    pub fn search_documents_in_subtree(&self, node_id: i32, query: &str) -> Result<Vec<Document>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(vec![]);
        }

        let node = self.get_tree_node_by_id(node_id)?;
        if node.location == Some("000".into()) {
            return Ok(vec![]);
        }

        let like = format!("%{}%", escape_like(query));
        let like_squished = format!("%{}%", escape_like(&squish(query)));
        let cast_content = if self.is_postgres { "convert_from(d.content, 'UTF8')" } else { "CAST(d.content AS TEXT)" };
        let sql = format!(
            "WITH RECURSIVE sub(id) AS (\
               SELECT ? \
               UNION ALL \
               SELECT tnl.child_node_id FROM tree_node_links tnl JOIN sub ON sub.id = tnl.parent_node_id\
             ) \
             SELECT DISTINCT d.hkap_id, d.variant_id, d.language_code, d.version, d.vehicle_component, \
             d.title, d.document_type, d.publication_date, d.file_format, \
             d.vehicle_component_with_document_index, d.new, d.bookmarked, d.content \
             FROM documents d \
             JOIN document_links dl ON dl.hkap_id = d.hkap_id \
             WHERE dl.node_id IN (SELECT id FROM sub) \
             AND (\
               d.title LIKE ? ESCAPE '\\' \
               OR d.vehicle_component_with_document_index LIKE ? ESCAPE '\\' \
               OR (d.file_format = 'xml' AND {cast_content} LIKE ? ESCAPE '\\') \
               OR (d.file_format = 'pdf' AND EXISTS (\
                 SELECT 1 FROM document_text dt \
                 WHERE dt.hkap_id = d.hkap_id \
                 AND (dt.text LIKE ? ESCAPE '\\' OR dt.text_squished LIKE ? ESCAPE '\\')\
               ))\
             )"
        );

        let sql = self.q(&sql);
        let mut results: Vec<Document> = block(
            sqlx::query_as::<_, Document>(&sql)
                .bind(node_id)
                .bind(&like)
                .bind(&like)
                .bind(&like)
                .bind(&like)
                .bind(&like_squished)
                .fetch_all(&self.pool)
        )?;
        results.sort();
        Ok(results)
    }

    pub fn get_children_nodes(&self, node_id: i32) -> Result<Vec<TreeNode>> {
        let q = self.q("SELECT tn.node_id, tn.vehicle, tn.year, tn.node_value, tn.name, \
                 tn.illustration_id, tn.location, tn.filter_applies \
                 FROM tree_nodes tn \
                 JOIN tree_node_links tnl ON tnl.child_node_id = tn.node_id \
                 WHERE tnl.parent_node_id = ? \
                 ORDER BY tn.location");
        let results: Vec<TreeNode> = block(
            sqlx::query_as::<_, TreeNode>(&q)
            .bind(node_id)
            .fetch_all(&self.pool)
        )?;
        Ok(results)
    }

    pub fn subtree_has_documents(&self, node_id: i32) -> Result<bool> {
        let q = self.q("WITH RECURSIVE sub(id) AS (\
                   SELECT node_id FROM tree_nodes WHERE node_id = ? \
                   UNION ALL \
                   SELECT tnl.child_node_id FROM tree_node_links tnl JOIN sub ON sub.id = tnl.parent_node_id\
                 ) \
                 SELECT COUNT(*) FROM document_links dl WHERE dl.node_id IN (SELECT id FROM sub)");
        let (cnt,): (i64,) = block(
            sqlx::query_as::<_, (i64,)>(&q)
            .bind(node_id)
            .fetch_one(&self.pool)
        )?;
        Ok(cnt > 0)
    }

    pub fn get_tree_node_by_id(&self, node_id: i32) -> Result<TreeNode> {
        let q = self.q("SELECT node_id, vehicle, year, node_value, name, illustration_id, location, filter_applies \
                 FROM tree_nodes WHERE node_id = ?");
        let result = block(
            sqlx::query_as::<_, TreeNode>(&q)
            .bind(node_id)
            .fetch_optional(&self.pool)
        )?;
        result.ok_or_else(|| anyhow::anyhow!(Error::NoSuchTreeNodeId(node_id)))
    }

    pub fn get_tool_data(&self, tool_id: &String) -> Result<Tool> {
        let q = self.q("SELECT id, title, tool_number_pag, tool_number_vw, tool_type, dealer_classification, \
                 description, hook_code, model_series, order_type, tool_order_number, state \
                 FROM tools WHERE tool_number_pag = ?");
        let result = block(
            sqlx::query_as::<_, Tool>(&q)
            .bind(tool_id)
            .fetch_optional(&self.pool)
        )?;
        result.ok_or_else(|| anyhow::anyhow!(Error::NoSuchTool(tool_id.clone())))
    }

    pub fn get_tool_distributors(&self, tool_id: i32) -> Result<Vec<ToolDistributor>> {
        let q = self.q("SELECT td.id, td.name, td.part_number, td.distributor_code, td.city, td.zip, \
                 td.street, td.phone, td.fax, td.email, td.web, td.creation_date, td.modification_date, td.key \
                 FROM tool_distributors td \
                 JOIN tool_distributors_links tdl ON tdl.tool_distributor_id = td.id \
                 WHERE tdl.tool_id = ?");
        let results: Vec<ToolDistributor> = block(
            sqlx::query_as::<_, ToolDistributor>(&q)
            .bind(tool_id)
            .fetch_all(&self.pool)
        )?;
        Ok(results)
    }

    pub fn get_referencing_documents(&self, year: i32, vehicle: String, tool_id: i32) -> Result<Vec<Document>> {
        let q = self.q("SELECT DISTINCT d.hkap_id, d.variant_id, d.language_code, d.version, d.vehicle_component, \
                 d.title, d.document_type, d.publication_date, d.file_format, \
                 d.vehicle_component_with_document_index, d.new, d.bookmarked, d.content \
                 FROM documents d \
                 JOIN document_links dl ON dl.hkap_id = d.hkap_id \
                 JOIN tree_nodes tn ON tn.node_id = dl.node_id \
                 WHERE tn.vehicle = ? AND tn.year = ? \
                 AND d.hkap_id IN (SELECT hkap_id FROM referencing_tool_documents WHERE tool_id = ?)");
        let results: Vec<Document> = block(
            sqlx::query_as::<_, Document>(&q)
            .bind(&vehicle)
            .bind(year)
            .bind(tool_id)
            .fetch_all(&self.pool)
        )?;
        Ok(results)
    }

    pub fn get_tree_node(&self, vehicle: &String, year: i32, location: Option<String>) -> Result<TreeNode> {
        let loc = location.clone().unwrap_or_else(|| "000".to_string());
        let q = self.q("SELECT node_id, vehicle, year, node_value, name, illustration_id, location, filter_applies \
                 FROM tree_nodes WHERE vehicle = ? AND year = ? AND location = ?");
        let result = block(
            sqlx::query_as::<_, TreeNode>(&q)
            .bind(vehicle)
            .bind(year)
            .bind(&loc)
            .fetch_optional(&self.pool)
        )?;
        result.ok_or_else(|| anyhow::anyhow!(Error::NoSuchTreeNode(location.unwrap_or_default())))
    }

    pub fn get_illustration(&self, illustration_id: i32) -> Result<String> {
        let q = self.q("SELECT content FROM illustrations WHERE illustration_id = ?");
        let result = block(
            sqlx::query_as::<_, (String,)>(&q)
            .bind(illustration_id)
            .fetch_optional(&self.pool)
        )?;
        result.map(|r| r.0).ok_or_else(|| anyhow::anyhow!(Error::NoSuchIllustration(illustration_id)))
    }

    pub fn get_tool_image(&self, image_id: &String) -> Result<Vec<u8>> {
        let q = self.q("SELECT content FROM tool_images WHERE id = ?");
        let result = block(
            sqlx::query_as::<_, (Vec<u8>,)>(&q)
            .bind(image_id)
            .fetch_optional(&self.pool)
        )?;
        result.map(|r| r.0).ok_or_else(|| anyhow::anyhow!(Error::NoSuchToolImage(image_id.clone())))
    }

    pub fn get_document(&self, hkap_id: &String) -> Result<Document> {
        let q = self.q("SELECT hkap_id, variant_id, language_code, version, vehicle_component, \
                 title, document_type, publication_date, file_format, \
                 vehicle_component_with_document_index, new, bookmarked, content \
                 FROM documents WHERE hkap_id = ?");
        let result = block(
            sqlx::query_as::<_, Document>(&q)
            .bind(hkap_id)
            .fetch_optional(&self.pool)
        )?;
        result.ok_or_else(|| anyhow::anyhow!(Error::NoSuchDocument(hkap_id.clone())))
    }

    pub fn get_documents(&self, hkap_ids: Vec<&String>) -> Result<Vec<Document>> {
        if hkap_ids.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: String = if self.is_postgres {
            hkap_ids.iter().enumerate().map(|(i, _)| format!("${}", i + 1)).collect::<Vec<_>>().join(",")
        } else {
            hkap_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")
        };
        let sql = format!(
            "SELECT hkap_id, variant_id, language_code, version, vehicle_component, \
             title, document_type, publication_date, file_format, \
             vehicle_component_with_document_index, new, bookmarked, content \
             FROM documents WHERE hkap_id IN ({})",
            placeholders
        );
        let mut q = sqlx::query_as::<_, Document>(&sql);
        for id in &hkap_ids {
            q = q.bind(id.as_str());
        }
        let mut results: Vec<Document> = block(q.fetch_all(&self.pool))?;
        results.sort();
        results.dedup();
        Ok(results)
    }

    pub fn get_media(&self, media_id: &String) -> Result<Vec<u8>> {
        let q = self.q("SELECT content FROM media_images WHERE id = ?");
        let result = block(
            sqlx::query_as::<_, (Vec<u8>,)>(&q)
            .bind(media_id)
            .fetch_optional(&self.pool)
        )?;
        result.map(|r| r.0).ok_or_else(|| anyhow::anyhow!(Error::NoSuchMedia(media_id.clone())))
    }

    pub fn get_workshop_image(&self, image_id: i32, size: &String) -> Result<Vec<u8>> {
        let q = self.q("SELECT content FROM workshop_images WHERE id = ? AND size = ?");
        let result = block(
            sqlx::query_as::<_, (Vec<u8>,)>(&q)
            .bind(image_id)
            .bind(size)
            .fetch_optional(&self.pool)
        )?;
        result.map(|r| r.0).ok_or_else(|| anyhow::anyhow!(Error::NoSuchImage(image_id)))
    }

    pub fn upsert_ui_texts(&self, ui_texts: &Vec<UiTexts>) -> Result<()> {
        for content in ui_texts.iter() {
            for (key, value) in content.translations.iter() {
                let q = self.q("INSERT INTO translations (key, value) VALUES (?, ?) \
                         ON CONFLICT(key) DO UPDATE SET value = excluded.value");
                block(
                    sqlx::query(&q)
                    .bind(key)
                    .bind(value)
                    .execute(&self.pool)
                )?;
            }
        }
        Ok(())
    }

    pub fn get_translations(&self) -> Result<HashMap<String, String>> {
        let results: Vec<Translations> = block(
            sqlx::query_as::<_, Translations>(
                "SELECT key, value FROM translations"
            )
            .fetch_all(&self.pool)
        )?;
        let mut map = HashMap::new();
        for t in results {
            map.insert(t.key, t.value);
        }
        Ok(map)
    }

    pub fn upsert_tree_node(&self, vehicle: &String, year: i32, node: &pcss::api_types::TreeNode) -> Result<()> {
        let q = self.q("INSERT INTO tree_nodes (node_id, vehicle, year, node_value, name, illustration_id, location, filter_applies) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(node_id) DO UPDATE SET \
                   vehicle = excluded.vehicle, year = excluded.year, node_value = excluded.node_value, \
                   name = excluded.name, illustration_id = excluded.illustration_id, \
                   location = excluded.location, filter_applies = excluded.filter_applies");
        block(
            sqlx::query(&q)
            .bind(node.node_id())
            .bind(vehicle)
            .bind(year)
            .bind(&node.node_value)
            .bind(&node.name)
            .bind(node.illustration_id)
            .bind(&node.location)
            .bind(node.filter_applies)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn upsert_tool(&self, tool: &pcss::api_types::Tool) -> Result<i32> {
        let q = self.q("INSERT INTO tools (title, tool_number_pag, tool_number_vw, tool_type, dealer_classification, \
                 description, hook_code, model_series, order_type, tool_order_number, state) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(tool_number_pag) DO UPDATE SET \
                   title = excluded.title, tool_number_vw = excluded.tool_number_vw, \
                   tool_type = excluded.tool_type, dealer_classification = excluded.dealer_classification, \
                   description = excluded.description, hook_code = excluded.hook_code, \
                   model_series = excluded.model_series, order_type = excluded.order_type, \
                   tool_order_number = excluded.tool_order_number, state = excluded.state \
                 RETURNING id");
        let row: (i32,) = block(
            sqlx::query_as::<_, (i32,)>(&q)
            .bind(&tool.title)
            .bind(&tool.tool_number_pag)
            .bind(&tool.tool_number_vw)
            .bind(&tool.tool_type)
            .bind(&tool.dealer_classification)
            .bind(&tool.description)
            .bind(&tool.hook_code)
            .bind(&tool.model_series)
            .bind(&tool.order_type)
            .bind(&tool.tool_order_number)
            .bind(&tool.state)
            .fetch_one(&self.pool)
        )?;

        let tool_id = row.0;

        for td in &tool.tool_distributors {
            let td_id = self.upsert_tool_distributor(td)?;
            let q = self.q("INSERT INTO tool_distributors_links (tool_id, tool_distributor_id) VALUES (?, ?) \
                     ON CONFLICT(tool_id, tool_distributor_id) DO NOTHING");
            block(
                sqlx::query(&q)
                .bind(tool_id)
                .bind(td_id)
                .execute(&self.pool)
            )?;
        }

        for ref_doc in &tool.referencing_documents {
            let q = self.q("INSERT INTO referencing_tool_documents (hkap_id, tool_id) VALUES (?, ?) \
                     ON CONFLICT(tool_id, hkap_id) DO NOTHING");
            block(
                sqlx::query(&q)
                .bind(&ref_doc.hkap_id)
                .bind(tool_id)
                .execute(&self.pool)
            )?;
        }

        Ok(tool_id)
    }

    pub fn upsert_tool_distributor(&self, td: &pcss::api_types::ToolDistributor) -> Result<i32> {
        let q = self.q("INSERT INTO tool_distributors \
                 (name, part_number, distributor_code, city, zip, street, phone, fax, email, web, creation_date, modification_date, key) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(name) DO UPDATE SET \
                   part_number = excluded.part_number, distributor_code = excluded.distributor_code, \
                   city = excluded.city, zip = excluded.zip, street = excluded.street, \
                   phone = excluded.phone, fax = excluded.fax, email = excluded.email, \
                   web = excluded.web, creation_date = excluded.creation_date, \
                   modification_date = excluded.modification_date, key = excluded.key \
                 RETURNING id");
        let row: (i32,) = block(
            sqlx::query_as::<_, (i32,)>(&q)
            .bind(&td.name)
            .bind(&td.part_number)
            .bind(&td.distributor_code)
            .bind(&td.city)
            .bind(&td.zip)
            .bind(&td.street)
            .bind(&td.phone)
            .bind(&td.fax)
            .bind(&td.email)
            .bind(&td.web)
            .bind(&td.creation_date)
            .bind(&td.modification_date)
            .bind(&td.key)
            .fetch_one(&self.pool)
        )?;
        Ok(row.0)
    }

    pub fn upsert_illustration(&self, illustration_id: i32, content: &String) -> Result<()> {
        let q = self.q("INSERT INTO illustrations (illustration_id, content) VALUES (?, ?) \
                 ON CONFLICT(illustration_id) DO UPDATE SET content = excluded.content");
        block(
            sqlx::query(&q)
            .bind(illustration_id)
            .bind(content)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn get_vehicle(&self, year: i32, vehicle: &String) -> Result<Vehicle> {
        let q = self.q("SELECT id, year, vehicle, name, image FROM vehicles WHERE year = ? AND vehicle = ?");
        let result = block(
            sqlx::query_as::<_, Vehicle>(&q)
            .bind(year)
            .bind(vehicle)
            .fetch_optional(&self.pool)
        )?;
        result.ok_or_else(|| anyhow::anyhow!(Error::NoSuchVehicle(year, vehicle.clone())))
    }

    pub fn get_vehicle_illustration(&self, year: i32, vehicle: &String) -> Result<Vec<u8>> {
        let v = self.get_vehicle(year, vehicle)?;
        Ok(v.image)
    }

    pub fn get_vehicles(&self) -> Result<Vec<Vehicle>> {
        let results: Vec<Vehicle> = block(
            sqlx::query_as::<_, Vehicle>(
                "SELECT id, year, vehicle, name, image FROM vehicles"
            )
            .fetch_all(&self.pool)
        )?;
        Ok(results)
    }

    pub fn store_vehicle(&self, vehicle: &settings::Vehicle) -> Result<()> {
        let path = Path::new(&self.vehicle_image_path).join(format!("{}-{}.png", vehicle.year, vehicle.vehicle));
        let image = fs::read(&path).unwrap_or_else(|_| {
            println!("note: no vehicle image found at {}", path.display());
            vec![]
        });

        self.upsert_vehicle_direct(&Vehicle {
            id: None,
            year: vehicle.year,
            vehicle: vehicle.vehicle.clone(),
            name: vehicle.name.clone(),
            image,
        })
    }

    pub fn upsert_document_text(&self, hkap_id: &str, text: &str) -> Result<()> {
        let squished = squish(text);
        let q = self.q("INSERT INTO document_text (hkap_id, text, text_squished) VALUES (?, ?, ?) \
                 ON CONFLICT(hkap_id) DO UPDATE SET text = excluded.text, text_squished = excluded.text_squished");
        block(
            sqlx::query(&q)
            .bind(hkap_id)
            .bind(text)
            .bind(squished)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn list_pdf_documents(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let rows: Vec<(String, Vec<u8>)> = block(
            sqlx::query_as::<_, (String, Vec<u8>)>(
                "SELECT hkap_id, content FROM documents WHERE file_format = 'pdf'"
            )
            .fetch_all(&self.pool)
        )?;
        Ok(rows)
    }

    pub fn count_document_text(&self) -> Result<i64> {
        let (n,): (i64,) = block(
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM document_text")
                .fetch_one(&self.pool)
        )?;
        Ok(n)
    }

    pub fn has_document_text(&self, hkap_id: &str) -> Result<bool> {
        let q = self.q("SELECT COUNT(*) FROM document_text WHERE hkap_id = ?");
        let (n,): (i64,) = block(
            sqlx::query_as::<_, (i64,)>(&q)
            .bind(hkap_id)
            .fetch_one(&self.pool)
        )?;
        Ok(n > 0)
    }

    pub fn upsert_media_image(&self, image_id: &String, image: &Vec<u8>) -> Result<()> {
        let q = self.q("INSERT INTO media_images (id, content) VALUES (?, ?) \
                 ON CONFLICT(id) DO UPDATE SET content = excluded.content");
        block(
            sqlx::query(&q)
            .bind(image_id)
            .bind(image)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    /// Concurrent batch upsert of media images. Runs up to DB_CONCURRENCY statements
    /// in flight against the pool, hiding per-statement round-trip latency.
    pub fn upsert_media_images(&self, images: Vec<(String, Vec<u8>)>) -> Result<()> {
        if images.is_empty() {
            return Ok(());
        }
        let q = self.q("INSERT INTO media_images (id, content) VALUES (?, ?) \
                 ON CONFLICT(id) DO UPDATE SET content = excluded.content");
        block(async {
            futures::stream::iter(images)
                .map(Ok::<_, sqlx::Error>)
                .try_for_each_concurrent(DB_CONCURRENCY, |(id, content)| {
                    let q = &q;
                    async move {
                        sqlx::query(q).bind(id).bind(content).execute(&self.pool).await.map(|_| ())
                    }
                })
                .await
        })?;
        Ok(())
    }

    /// Concurrent batch upsert of document full-text rows.
    pub fn upsert_document_texts(&self, texts: Vec<(String, String)>) -> Result<()> {
        if texts.is_empty() {
            return Ok(());
        }
        let q = self.q("INSERT INTO document_text (hkap_id, text, text_squished) VALUES (?, ?, ?) \
                 ON CONFLICT(hkap_id) DO UPDATE SET text = excluded.text, text_squished = excluded.text_squished");
        block(async {
            futures::stream::iter(texts)
                .map(Ok::<_, sqlx::Error>)
                .try_for_each_concurrent(DB_CONCURRENCY, |(hkap_id, text)| {
                    let q = &q;
                    async move {
                        let squished = squish(&text);
                        sqlx::query(q).bind(hkap_id).bind(text).bind(squished).execute(&self.pool).await.map(|_| ())
                    }
                })
                .await
        })?;
        Ok(())
    }


    pub fn upsert_tool_image(&self, image_id: &String, image: &Vec<u8>) -> Result<()> {
        let q = self.q("INSERT INTO tool_images (id, content) VALUES (?, ?) \
                 ON CONFLICT(id) DO UPDATE SET content = excluded.content");
        block(
            sqlx::query(&q)
            .bind(image_id)
            .bind(image)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn upsert_workshop_image(&self, id: i32, size: &String, image: &Vec<u8>) -> Result<()> {
        let q = self.q("INSERT INTO workshop_images (id, size, content) VALUES (?, ?, ?) \
                 ON CONFLICT(id, size) DO UPDATE SET content = excluded.content");
        block(
            sqlx::query(&q)
            .bind(id)
            .bind(size)
            .bind(image)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn link_document_to_node(&self, node_id: i32, hkap_id: &String) -> Result<()> {
        let q = self.q("INSERT INTO document_links (node_id, hkap_id) VALUES (?, ?) \
                 ON CONFLICT(node_id, hkap_id) DO NOTHING");
        block(
            sqlx::query(&q)
            .bind(node_id)
            .bind(hkap_id)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn upsert_document(&self, node_id: i32, document: &pcss::api_types::Document, raw_content: &Option<Vec<u8>>) -> Result<()> {
        let content = raw_content.clone().unwrap_or_default();
        let q = self.q("INSERT INTO documents (hkap_id, variant_id, language_code, version, vehicle_component, \
                 title, document_type, publication_date, file_format, vehicle_component_with_document_index, \
                 new, bookmarked, content) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(hkap_id) DO UPDATE SET \
                   variant_id = excluded.variant_id, language_code = excluded.language_code, \
                   version = excluded.version, vehicle_component = excluded.vehicle_component, \
                   title = excluded.title, document_type = excluded.document_type, \
                   publication_date = excluded.publication_date, file_format = excluded.file_format, \
                   vehicle_component_with_document_index = excluded.vehicle_component_with_document_index, \
                   new = excluded.new, bookmarked = excluded.bookmarked, content = excluded.content");
        block(
            sqlx::query(&q)
            .bind(&document.hkap_id)
            .bind(&document.variant_id)
            .bind(&document.language_code)
            .bind(document.version)
            .bind(&document.vehicle_component)
            .bind(&document.title)
            .bind(&document.document_type)
            .bind(&document.publication_date)
            .bind(&document.file_format)
            .bind(&document.vehicle_component_with_document_index)
            .bind(document.new)
            .bind(document.bookmarked)
            .bind(&content)
            .execute(&self.pool)
        )?;

        self.link_document_to_node(node_id, &document.hkap_id)?;
        Ok(())
    }

    pub fn upsert_tree_node_links(&self, parent: i32, child: i32) -> Result<()> {
        let q = self.q("INSERT INTO tree_node_links (parent_node_id, child_node_id) VALUES (?, ?) \
                 ON CONFLICT(parent_node_id, child_node_id) DO NOTHING");
        block(
            sqlx::query(&q)
            .bind(parent)
            .bind(child)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn insert_part(&self, vehicle: &String, year: i32, vehicle_component: &String, part: &pcss::api_types::Part) -> Result<()> {
        let q = self.q("INSERT INTO parts (vehicle, year, vehicle_component, part_id, paw_relevant, text) \
                 VALUES (?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(vehicle, year, vehicle_component, part_id) DO UPDATE SET \
                   paw_relevant = excluded.paw_relevant, text = excluded.text");
        block(
            sqlx::query(&q)
            .bind(vehicle)
            .bind(year)
            .bind(vehicle_component)
            .bind(&part.part_id)
            .bind(part.paw_relevant)
            .bind(&part.text)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn upsert_vehicle_direct(&self, vehicle: &Vehicle) -> Result<()> {
        if let Ok(existing) = self.get_vehicle(vehicle.year, &vehicle.vehicle) {
            let q = self.q("UPDATE vehicles SET name = ? WHERE id = ?");
            block(
                sqlx::query(&q)
                    .bind(&vehicle.name)
                    .bind(existing.id)
                    .execute(&self.pool)
            )?;
            if !vehicle.image.is_empty() {
                let q = self.q("UPDATE vehicles SET image = ? WHERE id = ?");
                block(
                    sqlx::query(&q)
                        .bind(&vehicle.image)
                        .bind(existing.id)
                        .execute(&self.pool)
                )?;
            }
            return Ok(());
        }
        let q = self.q("INSERT INTO vehicles (year, vehicle, name, image) VALUES (?, ?, ?, ?)");
        block(
            sqlx::query(&q)
            .bind(vehicle.year)
            .bind(&vehicle.vehicle)
            .bind(&vehicle.name)
            .bind(&vehicle.image)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn upsert_tree_node_direct(&self, node: &TreeNode) -> Result<()> {
        let q = self.q("INSERT INTO tree_nodes (node_id, vehicle, year, node_value, name, illustration_id, location, filter_applies) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(node_id) DO UPDATE SET \
                   vehicle = excluded.vehicle, year = excluded.year, node_value = excluded.node_value, \
                   name = excluded.name, illustration_id = excluded.illustration_id, \
                   location = excluded.location, filter_applies = excluded.filter_applies");
        block(
            sqlx::query(&q)
            .bind(node.node_id)
            .bind(&node.vehicle)
            .bind(node.year)
            .bind(&node.node_value)
            .bind(&node.name)
            .bind(node.illustration_id)
            .bind(&node.location)
            .bind(node.filter_applies)
            .execute(&self.pool)
        )?;
        Ok(())
    }

    pub fn upsert_document_direct(&self, document: &Document) -> Result<()> {
        let q = self.q("INSERT INTO documents (hkap_id, variant_id, language_code, version, vehicle_component, \
                 title, document_type, publication_date, file_format, vehicle_component_with_document_index, \
                 new, bookmarked, content) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
                 ON CONFLICT(hkap_id) DO UPDATE SET \
                   variant_id = excluded.variant_id, language_code = excluded.language_code, \
                   version = excluded.version, vehicle_component = excluded.vehicle_component, \
                   title = excluded.title, document_type = excluded.document_type, \
                   publication_date = excluded.publication_date, file_format = excluded.file_format, \
                   vehicle_component_with_document_index = excluded.vehicle_component_with_document_index, \
                   new = excluded.new, bookmarked = excluded.bookmarked, content = excluded.content");
        block(
            sqlx::query(&q)
            .bind(&document.hkap_id)
            .bind(&document.variant_id)
            .bind(&document.language_code)
            .bind(document.version)
            .bind(&document.vehicle_component)
            .bind(&document.title)
            .bind(&document.document_type)
            .bind(&document.publication_date)
            .bind(&document.file_format)
            .bind(&document.vehicle_component_with_document_index)
            .bind(document.new)
            .bind(document.bookmarked)
            .bind(&document.content)
            .execute(&self.pool)
        )?;
        Ok(())
    }
}
