use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::json;

use pcss::api_types::Response;
use pcss::workshop_literature::{DocumentType, FileFormat, LanguageCode, WorkshopLiterature};

use crate::content_store::ContentStore;
use crate::models::{Document, TreeNode, Vehicle};
use crate::settings::Settings;

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
pub struct LoadAudiArgs {
    /// Directory containing the raw Audi PDF source files (source_files/audi/<model>).
    /// The split step runs automatically via tools/r8_split.py when manifest.json is absent.
    #[arg(short = 'p', long, default_value = "source_files/audi/2018-R8")]
    pub path: String,

    /// Skip per-document PDF text extraction (search back-fill)
    #[arg(long, default_value_t = false)]
    pub no_text: bool,
}

/// If `path` contains raw PDFs but no manifest.json, run r8_split.py to produce
/// the split tree under `generated/` (temporary, regenerable output — kept out of
/// `source_files/`) and return that path. Otherwise return `path` as-is.
fn ensure_split(path: &str) -> Result<String> {
    let base = Path::new(path);
    if base.join("manifest.json").exists() {
        return Ok(path.to_string());
    }
    // Mirror the source layout under generated/, e.g.
    // source_files/audi/2018-R8 -> generated/audi/2018-R8-split
    let rel = path.strip_prefix("source_files/").unwrap_or(path);
    let split_path = format!("generated/{}-split", rel);
    println!("No manifest.json found — running r8_split.py to split PDFs into {} ...", split_path);
    let status = Command::new("python3")
        .args(["tools/r8_split.py", path, &split_path])
        .status()
        .context("failed to run tools/r8_split.py (is python3 available?)")?;
    if !status.success() {
        bail!("r8_split.py exited with status {}", status);
    }
    Ok(split_path)
}

#[derive(Deserialize)]
struct Manifest {
    vehicle: VehicleM,
    root: NodeM,
    nodes: Vec<NodeM>,
    documents: Vec<DocM>,
}

#[derive(Deserialize)]
struct VehicleM {
    year: i32,
    model: String,
    name: String,
}

#[derive(Deserialize)]
struct NodeM {
    node_id: i32,
    parent_node_id: Option<i32>,
    location: String,
    node_value: String,
    name: String,
}

#[derive(Deserialize)]
struct DocM {
    hkap_id: String,
    node_id: i32,
    parent_node_id: i32,
    location: String,
    node_value: String,
    title: String,
    vehicle_component: String,
    media_id: String,
    pdf: String,
}

impl DocM {
    // full section index shown in the workshop-literature list, e.g. "46.1"
    fn index(&self) -> &str {
        &self.node_value
    }
}

pub fn run(settings: &Settings, args: &LoadAudiArgs) -> Result<()> {
    let resolved_path = ensure_split(&args.path)?;
    let base = Path::new(&resolved_path);
    let manifest_path = base.join("manifest.json");
    if !manifest_path.exists() {
        bail!("manifest.json not found in {}", resolved_path);
    }

    let manifest: Manifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    let store = ContentStore::new(settings);
    let model = manifest.vehicle.model.clone();
    let year = manifest.vehicle.year;

    println!(
        "Loading Audi {} {} ({}) from {}",
        year, model, manifest.vehicle.name, resolved_path
    );

    // Vehicle row. Look for an image in the original source dir first (the split
    // dir is generated and shouldn't need manual files dropped in it), then fall
    // back to the split dir itself.
    let original_path = args.path.clone();
    let search_dirs = [original_path.as_str(), resolved_path.as_str()];
    let image = ["vehicle.png", "vehicle.jpg", "image.png", "image.jpg"]
        .iter()
        .flat_map(|n| search_dirs.iter().map(move |d| Path::new(d).join(n)))
        .find(|p| p.exists())
        .and_then(|p| fs::read(p).ok())
        .unwrap_or_default();
    if image.is_empty() {
        println!("note: no vehicle image found (drop vehicle.png in {} to set one)", original_path);
    }
    store.upsert_vehicle_direct(&Vehicle {
        id: None,
        year,
        vehicle: model.clone(),
        name: manifest.vehicle.name.clone(),
        image,
    })?;

    // Folder nodes: root, then main groups / repair groups / manual sub-nodes.
    let mk_node = |n: &NodeM| TreeNode {
        node_id: n.node_id,
        vehicle: Some(model.clone()),
        year: Some(year),
        node_value: n.node_value.clone(),
        name: Some(n.name.clone()),
        illustration_id: 0,
        location: Some(n.location.clone()),
        filter_applies: Some(false),
    };

    store.upsert_tree_node_direct(&mk_node(&manifest.root))?;
    for n in &manifest.nodes {
        store.upsert_tree_node_direct(&mk_node(n))?;
        if let Some(parent) = n.parent_node_id {
            store.upsert_tree_node_links(parent, n.node_id)?;
        }
    }

    // Documents: leaf node + media bytes + document row + link (+ text).
    let total = manifest.documents.len();
    let mut skipped = 0usize;

    // Fully prepared, ready-to-write document (built off-thread in the rayon pass).
    struct Prepared {
        media_id: String,
        content: Vec<u8>,
        document: Document,
        node_id: i32,
        hkap_id: String,
        text: Option<String>,
    }

    // Phase 1 (sequential, cheap): write the leaf tree nodes + links for every
    // document, and collect the ones that still need their content processed.
    let mut to_process: Vec<&DocM> = Vec::with_capacity(total);
    for d in &manifest.documents {
        let leaf = TreeNode {
            node_id: d.node_id,
            vehicle: Some(model.clone()),
            year: Some(year),
            node_value: d.node_value.clone(),
            name: Some(d.title.clone()),
            illustration_id: 0,
            location: Some(d.location.clone()),
            filter_applies: Some(false),
        };
        store.upsert_tree_node_direct(&leaf)?;
        store.upsert_tree_node_links(d.parent_node_id, d.node_id)?;

        if store.get_document(&d.hkap_id).is_ok() {
            skipped += 1;
        } else {
            to_process.push(d);
        }
    }
    println!("  {} folder/leaf nodes written; {} docs to process, {} already present",
        manifest.nodes.len() + 1 + total, to_process.len(), skipped);

    // Phase 2 (parallel, CPU/IO bound): read each PDF and extract its text off the
    // main thread, building the document payload. This is the expensive part.
    let prepared: Vec<Prepared> = to_process.par_iter().map(|d| {
        let pdf_path = base.join(&d.pdf);
        let content = fs::read(&pdf_path)
            .unwrap_or_else(|e| panic!("failed reading slice {}: {}", pdf_path.display(), e));
        let text = if args.no_text {
            None
        } else {
            crate::pdf_text::extract_pdf_text(&content).map(|t| t.trim().to_string())
        };
        let worklit = Response {
            payload: WorkshopLiterature {
                file_format: FileFormat::Pdf,
                language_code: LanguageCode::EnUs,
                hkap_id: d.hkap_id.clone(),
                variant_id: 1,
                version: None,
                latest_version: None,
                version_source_system: None,
                source_system: None,
                kdnr: d.vehicle_component.clone(),
                ti_number: None,
                publication_date: "2019-01-01T00:00:00.000+00:00".to_string(),
                modification_date: 0,
                title: d.title.clone(),
                file_name: format!("{}.pdf", d.hkap_id),
                document_type: DocumentType::Rm,
                target_hkap_id: None,
                content: None,
                toc: None,
                techvalues: None,
                mediacloud_image_ids: None,
                tools: None,
                media_cloud_file_id: Some(d.media_id.clone()),
                issue_date: None,
                vehicle_component_with_document_index: Some(d.index().to_string()),
                links: None,
                quality_line_segment: None,
                parts: None,
                laborpos: None,
                raw_content: None,
            },
            links: None,
        };
        let document = Document {
            hkap_id: d.hkap_id.clone(),
            variant_id: "1".to_string(),
            language_code: "en-us".to_string(),
            version: 1,
            vehicle_component: d.vehicle_component.clone(),
            title: d.title.clone(),
            document_type: "RM".to_string(),
            publication_date: worklit.payload.publication_date.clone(),
            file_format: "pdf".to_string(),
            vehicle_component_with_document_index: d.index().to_string(),
            new: false,
            bookmarked: false,
            content: serde_json::to_vec(&json!(worklit)).expect("serialize workshop literature"),
        };
        Prepared {
            media_id: d.media_id.clone(),
            content,
            document,
            node_id: d.node_id,
            hkap_id: d.hkap_id.clone(),
            text,
        }
    }).collect();

    // Phase 3 (batched, concurrent DB writes): document rows + links sequentially,
    // media bytes and full-text rows via concurrent batch upserts.
    for p in &prepared {
        store.upsert_document_direct(&p.document)?;
        store.link_document_to_node(p.node_id, &p.hkap_id)?;
    }
    let mut media = Vec::with_capacity(prepared.len());
    let mut texts = Vec::new();
    for p in prepared {
        media.push((p.media_id, p.content));
        if let Some(t) = p.text {
            texts.push((p.hkap_id, t));
        }
    }
    let extracted = texts.len();
    store.upsert_media_images(media)?;
    store.upsert_document_texts(texts)?;

    println!(
        "Done. {} documents ({} skipped), {} folder nodes, text extracted for {}/{}",
        total,
        skipped,
        manifest.nodes.len() + 1,
        extracted,
        total - skipped
    );
    Ok(())
}
