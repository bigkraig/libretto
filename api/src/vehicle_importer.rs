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
use pcss::workshop_literature::{Content, ContentType};
use crate::content_store::ContentStore;
use crate::settings::Settings;

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

    pub fn import<'a>(&self, vehicle: &String, year: i32) -> Result<&VehicleImporter> {
        let mut pcss_client = PCSS::new(&self.settings.importer.cache_dir, &self.settings.importer.cookie);
        let content_store = ContentStore::new(&self.settings);
        let mut tree_nodes = pcss_client.get_tree_nodes(&vehicle, year)?;
        let mut bad_tools = vec![];
        let mut good_tools = vec![];

        let ui_texts = pcss_client.get_ui_texts()?;
        content_store.upsert_ui_texts(&ui_texts)?;

        // Vehicle-scoped write buffers, flushed in chunks. Media upserts go out as one
        // concurrent batch and PDF text extraction runs as one big rayon pass — per-node
        // batches were too small to parallelize (most nodes have 0-1 PDFs), so extraction
        // ran nearly sequentially. Chunking bounds peak memory.
        let mut media_buf: Vec<(String, Vec<u8>)> = Vec::new();
        let mut pdf_buf: Vec<(String, Vec<u8>)> = Vec::new();
        const FLUSH_AT: usize = 96;

        for node in &mut tree_nodes {
            // hooohooo hacks! the 991.2 gt3 transmission node does not have an illustration_id set, but it should.
            if node.node_id() == 29052 { node.illustration_id = 3708 }

            let all_literature = pcss_client.list_workshop_literature(&vehicle, year, &node.node_value)?;

            println!("Storing {} {} {}: {:?}", &vehicle, &node.node_value, node.clone().name.unwrap_or_default(), all_literature.len());
            content_store.upsert_tree_node(&vehicle, year, node)?;

            for link in pcss_client.get_children_ids(&node)? {
                content_store.upsert_tree_node_links(node.node_id(), link)?;
            }

            for part_id in pcss_client.get_parts(&vehicle, year, &node.node_value)? {
                content_store.insert_part(&vehicle, year, &node.node_value, &part_id)?;
            }

            if node.illustration_id != 0 {
                let illustration = pcss_client.get_illustration(node.illustration_id)?;
                let patched_illustration = patch_illustration(illustration)?;
                content_store.upsert_illustration(node.illustration_id, &patched_illustration)?;
            }

            for document in all_literature {
                // These return 500 errors
                if document.hkap_id.eq("81372188") { // Taycan
                    continue;
                }
                if document.hkap_id.eq("75046626") { // 991
                    continue;
                }

                let worklit = pcss_client.get_workshop_literature(&vehicle, year, &document)?;
                content_store.upsert_document(node.node_id(), &document, &worklit.raw_content.clone())?;
                content_store.link_document_to_node(node.node_id(), &document.hkap_id)?;

                // Get media images from document
                if let Some(media_images) = &worklit.mediacloud_image_ids {
                    let mut media_image_ids: Vec<&String> = vec![];
                    media_image_ids.append(&mut media_images.mediacloud_small.iter().collect::<Vec<&String>>());
                    media_image_ids.append(&mut media_images.mediacloud_normal.iter().collect::<Vec<&String>>());
                    media_image_ids.sort();
                    media_image_ids.dedup();

                    if media_image_ids.len() != 0 {
                        let media_images = pcss_client.get_media_ids(media_image_ids)?;
                        for (image_id, content) in media_images {
                            content_store.upsert_media_image(&image_id, &content)?;
                        }
                    }
                }

                // Known bad tools
                bad_tools.push("P90019".to_string());
                bad_tools.push("V.A.G 1866/1".to_string());
                bad_tools.push("V.A.G 1866".to_string());
                bad_tools.push("VAS 6446A".to_string());
                bad_tools.push("VAS 6446/10".to_string());
                bad_tools.push("VAS 6446/11".to_string());
                bad_tools.push("1139".to_string());
                bad_tools.push("Nr.168".to_string());

                // Get Tools from the document
                if let Some(tool_content) = &worklit.tools {
                    for tool in tool_content {
                        if bad_tools.contains(&tool.tool_number) || good_tools.contains(&tool.tool_number) {
                            continue;
                        }
                        let tool_data = pcss_client.get_tool_data(&tool.tool_number).context(format!("failed getting tool data {}", &tool.tool_number))?;
                        content_store.upsert_tool(&tool_data)?;
                        let image = pcss_client.get_tool(&tool.tool_number).context(format!("failed getting tool {}", &tool.tool_number))?;
                        content_store.upsert_tool_image(&tool.tool_number, &image)?;
                        good_tools.push(tool.tool_number.clone());
                    }
                }


                // this includes PDFs
                if let Some(media_cloud_file_id) = &worklit.media_cloud_file_id {
                    let content = pcss_client.get_pdf(media_cloud_file_id).context(format!("failed getting media_cloud_file_id {}", &media_cloud_file_id))?;
                    if document.file_format == "pdf" {
                        pdf_buf.push((document.hkap_id.clone(), content.clone()));
                    }
                    media_buf.push((media_cloud_file_id.to_string(), content));
                }

                // Get Images from the document
                for item in &worklit.pick_children(&vec![ContentType::Image]) {
                    match item {
                        Content::Image(image) => {
                            // Get media images
                            let mut media_image_ids = vec![];
                            if image.mediacloud_normal.len() != 0 {
                                media_image_ids.push(&image.mediacloud_normal);
                            }
                            if image.mediacloud_large.len() != 0 {
                                media_image_ids.push(&image.mediacloud_large);
                            }
                            media_image_ids.sort();
                            media_image_ids.dedup();
                            if media_image_ids.len() != 0 {
                                let media_images = pcss_client.get_media_ids(media_image_ids)?;
                                for (image_id, content) in media_images {
                                    media_buf.push((image_id, content));
                                }
                            }

                            // Get image key
                            if &image.key == "" {
                                continue;
                            }

                            for size in vec!["normal", "large"] {
                                let img = pcss_client.get_workshop_image(&image.key, size)?;
                                content_store.upsert_workshop_image(std::str::FromStr::from_str(&image.key)?, &size.to_string(), &img)?;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Flush in chunks to bound memory while keeping the rayon batch large
            // enough to saturate all cores.
            if media_buf.len() + pdf_buf.len() >= FLUSH_AT {
                flush_writes(&content_store, std::mem::take(&mut media_buf), std::mem::take(&mut pdf_buf))?;
            }
        }

        // Flush whatever's left after the last node.
        flush_writes(&content_store, media_buf, pdf_buf)?;

        Ok(self)
    }
}

/// Batch-write buffered media bytes (concurrent) and extract PDF text in parallel
/// across all cores, then batch-write the text rows (concurrent).
fn flush_writes(
    store: &ContentStore,
    media: Vec<(String, Vec<u8>)>,
    pdfs: Vec<(String, Vec<u8>)>,
) -> Result<()> {
    store.upsert_media_images(media)?;
    let texts: Vec<(String, String)> = pdfs
        .par_iter()
        .filter_map(|(hkap_id, content)| {
            crate::pdf_text::extract_pdf_text(content)
                .map(|t| (hkap_id.clone(), collapse_whitespace(&t)))
        })
        .collect();
    store.upsert_document_texts(texts)?;
    Ok(())
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