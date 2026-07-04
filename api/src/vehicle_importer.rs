use std::io::Cursor;

use anyhow::{Context, Result};
use clap::arg;
use rayon::prelude::*;
use quick_xml::events::{BytesCData, BytesStart};
use quick_xml::events::attributes::Attribute;
use quick_xml::name::QName;
use quick_xml::Writer;
use thiserror::Error;

use pcss::PCSS;
use pcss::api_types::{Document, Part, Tool, TreeNode};
use pcss::workshop_literature::{Content, ContentType};
use crate::content_store::ContentStore;
use crate::settings::Settings;

/// Everything fetched for a single tree node, materialized in the parallel fetch
/// phase so the load phase can write it without touching the PCSS client.
struct NodeData {
    node: TreeNode,
    child_ids: Vec<i32>,
    parts: Vec<Part>,
    illustration: Option<(i32, String)>,
    documents: Vec<DocData>,
}

struct DocData {
    node_id: i32,
    document: Document,
    raw_content: Option<Vec<u8>>,
    /// media_images rows: (id, bytes) — PDF + mediacloud images
    media: Vec<(String, Vec<u8>)>,
    /// (hkap_id, pdf bytes) queued for full-text extraction
    pdf_text: Option<(String, Vec<u8>)>,
    /// (tool_data, tool_number, image bytes)
    tools: Vec<(Tool, String, Vec<u8>)>,
    /// (illustration key, size, image bytes) for workshop_images
    workshop_images: Vec<(i32, String, Vec<u8>)>,
}

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
        let pcss_client = PCSS::new(&self.settings.importer.cache_dir, &self.settings.importer.cookie);
        let content_store = ContentStore::new(&self.settings);
        let mut tree_nodes = pcss_client.get_tree_nodes(&vehicle, year)?;

        let ui_texts = pcss_client.get_ui_texts()?;
        content_store.upsert_ui_texts(&ui_texts)?;

        // hooohooo hacks! the 991.2 gt3 transmission node has no illustration_id set.
        for node in &mut tree_nodes {
            if node.node_id() == 29052 { node.illustration_id = 3708; }
        }

        // Two-phase, chunked: FETCH each chunk of nodes in parallel (PCSS cache reads +
        // parsing → NodeData, no DB, no shared state), then LOAD the chunk (DB writes, no
        // PCSS). Chunking bounds peak memory (media/PDF bytes held before writing).
        const CHUNK_NODES: usize = 24;
        let total = tree_nodes.len();
        let mut done = 0usize;
        for chunk in tree_nodes.chunks(CHUNK_NODES) {
            let fetched: Vec<NodeData> = chunk
                .par_iter()
                .map(|node| self.fetch_node(&pcss_client, vehicle, year, node))
                .collect::<Result<Vec<_>>>()?;
            self.load_nodes(&content_store, vehicle, year, fetched, extract_text)?;
            done += chunk.len();
            println!("  loaded {}/{} nodes", done, total);
        }

        Ok(self)
    }

    /// FETCH phase (runs on the rayon pool): pull everything for one node out of the
    /// PCSS cache into owned structures. No DB access, no shared mutable state.
    fn fetch_node(&self, pcss: &PCSS, vehicle: &String, year: i32, node: &TreeNode) -> Result<NodeData> {
        let all_literature = pcss.list_workshop_literature(vehicle, year, &node.node_value)?;
        let child_ids = pcss.get_children_ids(node)?;
        let parts = pcss.get_parts(vehicle, year, &node.node_value)?;
        let illustration = if node.illustration_id != 0 {
            let ill = pcss.get_illustration(node.illustration_id)?;
            Some((node.illustration_id, patch_illustration(ill)?))
        } else {
            None
        };

        let mut documents = Vec::new();
        for document in all_literature {
            // These return 500 errors
            if document.hkap_id == "81372188" || document.hkap_id == "75046626" {
                continue;
            }

            let worklit = pcss.get_workshop_literature(vehicle, year, &document)?;
            let mut media: Vec<(String, Vec<u8>)> = Vec::new();

            if let Some(mci) = &worklit.mediacloud_image_ids {
                let mut ids: Vec<&String> = Vec::new();
                ids.extend(mci.mediacloud_small.iter());
                ids.extend(mci.mediacloud_normal.iter());
                ids.sort();
                ids.dedup();
                if !ids.is_empty() {
                    media.extend(pcss.get_media_ids(ids)?);
                }
            }

            let mut tools = Vec::new();
            if let Some(tool_content) = &worklit.tools {
                for tool in tool_content {
                    if is_bad_tool(&tool.tool_number) {
                        continue;
                    }
                    let tool_data = pcss.get_tool_data(&tool.tool_number)
                        .context(format!("failed getting tool data {}", &tool.tool_number))?;
                    let image = pcss.get_tool(&tool.tool_number)
                        .context(format!("failed getting tool {}", &tool.tool_number))?;
                    tools.push((tool_data, tool.tool_number.clone(), image));
                }
            }

            let mut pdf_text = None;
            if let Some(media_cloud_file_id) = &worklit.media_cloud_file_id {
                let content = pcss.get_pdf(media_cloud_file_id)
                    .context(format!("failed getting media_cloud_file_id {}", &media_cloud_file_id))?;
                if document.file_format == "pdf" {
                    pdf_text = Some((document.hkap_id.clone(), content.clone()));
                }
                media.push((media_cloud_file_id.to_string(), content));
            }

            let mut workshop_images = Vec::new();
            for item in &worklit.pick_children(&vec![ContentType::Image]) {
                if let Content::Image(image) = item {
                    let mut ids = vec![];
                    if image.mediacloud_normal.len() != 0 { ids.push(&image.mediacloud_normal); }
                    if image.mediacloud_large.len() != 0 { ids.push(&image.mediacloud_large); }
                    ids.sort();
                    ids.dedup();
                    if !ids.is_empty() {
                        media.extend(pcss.get_media_ids(ids)?);
                    }
                    if image.key.is_empty() {
                        continue;
                    }
                    for size in ["normal", "large"] {
                        let img = pcss.get_workshop_image(&image.key, size)?;
                        workshop_images.push((std::str::FromStr::from_str(&image.key)?, size.to_string(), img));
                    }
                }
            }

            documents.push(DocData {
                node_id: node.node_id(),
                document,
                raw_content: worklit.raw_content,
                media,
                pdf_text,
                tools,
                workshop_images,
            });
        }

        Ok(NodeData {
            node: node.clone(),
            child_ids,
            parts,
            illustration,
            documents,
        })
    }

    /// LOAD phase: write a chunk of fetched nodes. Structural rows go out sequentially;
    /// media bytes and PDF text are batched (concurrent upserts + parallel extraction).
    fn load_nodes(
        &self,
        store: &ContentStore,
        vehicle: &String,
        year: i32,
        nodes: Vec<NodeData>,
        extract_text: bool,
    ) -> Result<()> {
        for nd in &nodes {
            store.upsert_tree_node(vehicle, year, &nd.node)?;
            for link in &nd.child_ids {
                store.upsert_tree_node_links(nd.node.node_id(), *link)?;
            }
            for part in &nd.parts {
                store.insert_part(vehicle, year, &nd.node.node_value, part)?;
            }
            if let Some((id, content)) = &nd.illustration {
                store.upsert_illustration(*id, content)?;
            }
            for doc in &nd.documents {
                store.upsert_document(doc.node_id, &doc.document, &doc.raw_content)?;
                store.link_document_to_node(doc.node_id, &doc.document.hkap_id)?;
                for (tool_data, number, image) in &doc.tools {
                    store.upsert_tool(tool_data)?;
                    store.upsert_tool_image(number, image)?;
                }
                for (key, size, img) in &doc.workshop_images {
                    store.upsert_workshop_image(*key, size, img)?;
                }
            }
        }

        // Consume the chunk for the batched media + text writes (moves bytes, no clone).
        let mut media = Vec::new();
        let mut pdfs = Vec::new();
        for nd in nodes {
            for doc in nd.documents {
                media.extend(doc.media);
                if let Some(pt) = doc.pdf_text {
                    pdfs.push(pt);
                }
            }
        }
        store.upsert_media_images(media)?;
        if extract_text {
            let texts: Vec<(String, String)> = pdfs
                .par_iter()
                .filter_map(|(hkap_id, content)| {
                    crate::pdf_text::extract_pdf_text(content)
                        .map(|t| (hkap_id.clone(), collapse_whitespace(&t)))
                })
                .collect();
            store.upsert_document_texts(texts)?;
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