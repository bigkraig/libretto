use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use clap::arg;
use dashmap::DashSet;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use quick_xml::events::{BytesCData, BytesStart};
use quick_xml::events::attributes::Attribute;
use quick_xml::name::QName;
use quick_xml::Writer;
use thiserror::Error;

use pcss::PCSS;
use pcss::api_types::TreeNode;
use pcss::workshop_literature::{Content, ContentType};
use crate::content_store::ContentStore;
use crate::settings::Settings;

/// How many nodes to fetch+write in parallel. Each worker reads, writes, and frees a node's
/// data on its own thread (no cross-thread hand-off of image buffers — that pattern thrashed
/// the allocator), so this also bounds concurrent DB connections; keep it at/under the pool
/// size (`DB_CONCURRENCY + 2`).
const IMPORT_WORKERS: usize = 16;

/// How many media ids to read per `get_media_ids` call. That call loads all the bytes for a
/// batch into memory before returning, so keeping the batch small bounds a worker's footprint
/// when a single document references a huge number of images.
const MEDIA_READ_CHUNK: usize = 8;

/// Tools that return errors from PCSS and should be skipped.
fn is_bad_tool(number: &str) -> bool {
    const BAD: &[&str] = &[
        "P90019", "V.A.G 1866/1", "V.A.G 1866", "VAS 6446A",
        "VAS 6446/10", "VAS 6446/11", "1139", "Nr.168",
    ];
    BAD.contains(&number)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("too many roots received")]
    TooManyRoots,
}


#[derive(clap::Args)]
#[command(version, about, long_about = None)]
pub struct VehicleImporterArgs {
    /// Vehicle model to scrape, 991810, 981810, Y11BFH1, etc
    #[arg(short = 'm')]
    pub model: Option<String>,

    /// Model Year
    #[arg(short = 'y')]
    pub year: Option<i32>,

    /// Skip PDF full-text extraction (the slow, CPU-heavy search back-fill).
    /// Run it separately later with `extract-pdf-text`.
    #[arg(long, default_value_t = false)]
    pub no_text: bool,
}


pub struct VehicleImporter {
    settings: Settings,
}

impl VehicleImporter {
    pub fn new(settings: &Settings) -> Self {
        VehicleImporter {
            settings: settings.clone(),
        }
    }

    pub fn import<'a>(&self, vehicle: &String, year: i32, extract_text: bool) -> Result<&VehicleImporter> {
        let start = std::time::Instant::now();
        let pcss_client = PCSS::new(&self.settings.importer.cache_dir, &self.settings.importer.cookie);
        let content_store = ContentStore::new(&self.settings);
        let mut tree_nodes = pcss_client.get_tree_nodes(&vehicle, year)?;

        let ui_texts = pcss_client.get_ui_texts()?;
        content_store.upsert_ui_texts(&ui_texts)?;

        // hooohooo hacks! the 991.2 gt3 transmission node has no illustration_id set.
        for node in &mut tree_nodes {
            if node.node_id() == 29052 { node.illustration_id = 3708; }
        }

        // Parallel per-node import. Each worker fetches ONE node and writes all of its rows and
        // images itself, reading → writing → freeing each blob on its own thread. There is no
        // hand-off of image buffers between threads: allocating on a producer and freeing on a
        // consumer (the previous design) made every allocator spend the bulk of its time on
        // cross-thread frees, which is what made the image-heavy nodes pathologically slow.
        // Parallelism across nodes provides the throughput instead. A bounded rayon pool caps
        // concurrent DB connections.
        let total = tree_nodes.len();

        // Vehicle-wide dedup of media/image reads. `media_images`/`workshop_images` are global
        // tables keyed by id, and a vehicle's sibling nodes reference the same illustrations
        // over and over, so reading each unique file once across the whole vehicle (not once
        // per node) avoids redundant cache reads and writes. Shared across the workers.
        let seen_media: DashSet<String> = DashSet::new();
        let seen_ws: DashSet<(String, &'static str)> = DashSet::new();
        let seen_tools: DashSet<String> = DashSet::new();
        let done = AtomicUsize::new(0);

        // Large per-worker stack: pdf-extract can recurse deeply on some PDFs and overflow the
        // default rayon stack. Lazily committed, so it costs no real memory unless used.
        let pool = ThreadPoolBuilder::new()
            .num_threads(IMPORT_WORKERS)
            .stack_size(256 * 1024 * 1024)
            .build()?;
        pool.install(|| {
            tree_nodes.par_iter().try_for_each(|node| -> Result<()> {
                self.process_node(&pcss_client, &content_store, vehicle, year, node,
                    &seen_media, &seen_ws, &seen_tools, extract_text)?;
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                if n % 24 == 0 || n == total {
                    println!("  loaded {}/{} nodes ({:.0}s)", n, total, start.elapsed().as_secs_f64());
                }
                Ok(())
            })
        })?;

        println!(
            "  {} {}: {} nodes in {:.1}s",
            vehicle, year, total, start.elapsed().as_secs_f64()
        );
        Ok(self)
    }

    /// Fetch one node from the PCSS cache and write all of it to the DB, on this worker thread.
    /// Every image is read, written, and freed here — nothing is handed to another thread — so
    /// allocation and deallocation stay thread-local.
    fn process_node(
        &self,
        pcss: &PCSS,
        store: &ContentStore,
        vehicle: &String,
        year: i32,
        node: &TreeNode,
        seen_media: &DashSet<String>,
        seen_ws: &DashSet<(String, &'static str)>,
        seen_tools: &DashSet<String>,
        extract_text: bool,
    ) -> Result<()> {
        let all_literature = pcss.list_workshop_literature(vehicle, year, &node.node_value)?;

        store.upsert_tree_node(vehicle, year, node)?;
        for link in pcss.get_children_ids(node)? {
            store.upsert_tree_node_links(node.node_id(), link)?;
        }
        for part in pcss.get_parts(vehicle, year, &node.node_value)? {
            store.insert_part(vehicle, year, &node.node_value, &part)?;
        }
        if node.illustration_id != 0 {
            let ill = pcss.get_illustration(node.illustration_id)?;
            store.upsert_illustration(node.illustration_id, &patch_illustration(ill)?)?;
        }
        let node_id = node.node_id();

        // `seen_media`/`seen_ws`/`seen_tools` are shared across the whole vehicle (see `import`),
        // so a file referenced by many documents — within this node or any sibling node — is
        // read and written exactly once.
        for document in all_literature {
            // These return 500 errors
            if document.hkap_id == "81372188" || document.hkap_id == "75046626" {
                continue;
            }

            let worklit = pcss.get_workshop_literature(vehicle, year, &document)?;
            store.upsert_document(node_id, &document, &worklit.raw_content)?;

            // Read each image/PDF, write it, and let it drop before the next — bounded footprint,
            // and every allocation is freed on this same worker thread.
            if let Some(mci) = &worklit.mediacloud_image_ids {
                let mut ids: Vec<&String> = Vec::new();
                ids.extend(mci.mediacloud_small.iter());
                ids.extend(mci.mediacloud_normal.iter());
                ids.retain(|id| seen_media.insert((*id).clone()));
                for chunk in ids.chunks(MEDIA_READ_CHUNK) {
                    for (id, content) in pcss.get_media_ids(chunk.to_vec())? {
                        store.upsert_media_image(&id, &content)?;
                    }
                }
            }

            if let Some(tool_content) = &worklit.tools {
                for tool in tool_content {
                    if is_bad_tool(&tool.tool_number) {
                        continue;
                    }
                    if !seen_tools.insert(tool.tool_number.clone()) {
                        continue;
                    }
                    let tool_data = pcss.get_tool_data(&tool.tool_number)
                        .context(format!("failed getting tool data {}", &tool.tool_number))?;
                    store.upsert_tool(&tool_data)?;
                    let image = pcss.get_tool(&tool.tool_number)
                        .context(format!("failed getting tool {}", &tool.tool_number))?;
                    store.upsert_tool_image(&tool.tool_number, &image)?;
                }
            }

            if let Some(media_cloud_file_id) = &worklit.media_cloud_file_id {
                let content = pcss.get_pdf(media_cloud_file_id)
                    .context(format!("failed getting media_cloud_file_id {}", &media_cloud_file_id))?;
                if extract_text && document.file_format == "pdf" {
                    if let Some(text) = crate::pdf_text::extract_pdf_text(&content) {
                        let normalized = collapse_whitespace(&text);
                        if let Err(e) = store.upsert_document_text(&document.hkap_id, &normalized) {
                            eprintln!("warning: failed to store extracted text for {}: {}", &document.hkap_id, e);
                        }
                    }
                }
                if seen_media.insert(media_cloud_file_id.to_string()) {
                    store.upsert_media_image(&media_cloud_file_id.to_string(), &content)?;
                }
            }

            for item in &worklit.pick_children(&vec![ContentType::Image]) {
                if let Content::Image(image) = item {
                    let mut ids = vec![];
                    if image.mediacloud_normal.len() != 0 { ids.push(&image.mediacloud_normal); }
                    if image.mediacloud_large.len() != 0 { ids.push(&image.mediacloud_large); }
                    ids.retain(|id| seen_media.insert((**id).clone()));
                    for chunk in ids.chunks(MEDIA_READ_CHUNK) {
                        for (id, content) in pcss.get_media_ids(chunk.to_vec())? {
                            store.upsert_media_image(&id, &content)?;
                        }
                    }
                    if image.key.is_empty() {
                        continue;
                    }
                    for size in ["normal", "large"] {
                        if !seen_ws.insert((image.key.clone(), size)) {
                            continue;
                        }
                        let img = pcss.get_workshop_image(&image.key, size)?;
                        store.upsert_workshop_image(std::str::FromStr::from_str(&image.key)?, &size.to_string(), &img)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_was_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(ch);
            last_was_space = false;
        }
    }
    out.trim().to_string()
}

fn patch_illustration(illustration: String) -> Result<String> {
    let mut reader = quick_xml::reader::Reader::from_str(illustration.as_str());
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    use quick_xml::events::Event;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break, // exits the loop when reaching end of file

            // Delete the width/height from the svg
            Ok(Event::Start(e)) if e.name().as_ref() == b"svg" => {
                let attributes = e.attributes()
                    .map(|attr| attr.unwrap())
                    .filter(|attr| { attr.key != QName(b"width") && attr.key != QName(b"height") }).collect::<Vec<Attribute>>();
                let elem = BytesStart::new("svg").with_attributes(attributes);
                assert!(writer.write_event(Event::Start(elem)).is_ok())
            }

            // Style patches
            Ok(Event::Start(e)) if e.name().as_ref() == b"style" => {
                assert!(writer.write_event(Event::Start(e)).is_ok());
                loop {
                    match reader.read_event() {
                        Ok(Event::End(elem)) => {
                            assert!(writer.write_event(Event::End(elem)).is_ok());
                            break;
                        }

                        // Fix style on some images, like 3706
                        Ok(Event::CData(data)) => {
                            let mut cdata = String::from_utf8(Vec::from(data.clone().into_inner()))?;
                            if !cdata.contains("fill-opacity:0.011765") {
                                cdata.push_str(" .fil99 {fill:white;fill-opacity:0.011765}\r\n");
                            }
                            assert!(writer.write_event(Event::CData(BytesCData::new(cdata))).is_ok());
                        }

                        Err(elem) => panic!("Error at position {}: {:?}", reader.error_position(), elem),
                        // Ok(e) => println!("{:?}", e),
                        Ok(event) => assert!(writer.write_event(event).is_ok()),
                    }
                }
            }

            // Locate all <g id=CC_HOTSPOT> for class patching on some images, like 3706
            Ok(Event::Start(ref e)) if e.clone().name().as_ref() == b"g" => {
                if let Some(id) = match &e.try_get_attribute(b"id") {
                    Ok(Some(a)) => match String::from_utf8(a.value.clone().to_vec()) {
                        Ok(a) => Some(a),
                        _ => None,
                    }
                    _ => None,
                } {
                    match id.as_str() {
                        "CC_HOTSPOT" => {
                            assert!(writer.write_event(Event::Start(e.clone())).is_ok());
                            let type_to_patch: Vec<&[u8]> = vec![b"path", b"polygon", b"rect"];
                            loop {
                                match reader.read_event() {
                                    Ok(Event::Empty(ref e))  if type_to_patch.contains(&e.clone().name().as_ref()) => {
                                        let mut elem = e.to_owned();

                                        let new_attrs = elem.attributes()
                                            .filter_map(|a| a.ok())
                                            .map(|attr| {
                                                let mut value = String::from_utf8(attr.value.to_vec()).expect("Invalid UTF-8");
                                                match attr.key {
                                                    QName(b"class") => {
                                                        value.push_str(" fil99");
                                                    }
                                                    _ => {}
                                                }
                                                (String::from_utf8(attr.key.as_ref().to_vec()).expect("Invalid UTF-8"), value)
                                            }).collect::<Vec<(String, String)>>();

                                        elem.clear_attributes();
                                        for attr in new_attrs {
                                            elem.push_attribute((attr.0.as_str(), attr.1.as_str()));
                                        }

                                        assert!(writer.write_event(Event::Empty(elem)).is_ok())
                                    }

                                    Ok(Event::End(elem)) => {
                                        assert!(writer.write_event(Event::End(elem)).is_ok());
                                        break;
                                    }

                                    Err(elem) => panic!("Error at position {}: {:?}", reader.error_position(), elem),
                                    Ok(event) => assert!(writer.write_event(event).is_ok()),
                                }
                            }
                        }
                        _ => assert!(writer.write_event(Event::Start(e.clone())).is_ok())
                    }
                }
            }

            // Ok(e) => println!("{:?}", e),
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(e) => assert!(writer.write_event(e).is_ok()),
        }
    }
    let result: Vec<u8> = writer.into_inner().into_inner();
    let s = String::from_utf8(result)?;
    return Ok(s);
}