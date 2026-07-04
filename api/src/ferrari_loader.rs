use std::fs;
use std::io::Cursor;
use std::path::Path;

use uuid::Uuid;
use anyhow::{bail, Result};
use lopdf::{Document as PdfDocument, Object, ObjectId};
use regex::Regex;
use serde_json::json;
use pcss::api_types::Response;
use pcss::workshop_literature::{DocumentType, FileFormat, LanguageCode, WorkshopLiterature};
use crate::content_store::ContentStore;
use crate::models::{Document, TreeNode, Vehicle};
use crate::settings::Settings;


const FERRARI_LINK_PREFIX: &'static str = "https://modiscs.ferrari.it/techdoc_techinfo/";
const ROOT: &'static str = "https://libretto.bigkraig.com";
// const ROOT: &'static str = "http://localhost:3000";

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
pub struct LoadFerrariArgs {
    /// Root path to Ferrari PDF documents (e.g., ferrari/2017-F151M)
    #[arg(short = 'p', long)]
    pub path: String,

    /// Model year (e.g., 2017)
    #[arg(short = 'y', long)]
    pub year: i32,

    /// Vehicle model code (e.g., F151M)
    #[arg(short = 'm', long)]
    pub model: String,
}

pub struct FerrariLoader {
    content_store: ContentStore,
}

impl FerrariLoader {
    pub fn new(content_store: ContentStore) -> Self {
        FerrariLoader { content_store }
    }

    pub fn load(&self, args: &LoadFerrariArgs, settings: &Settings) -> Result<()> {
        let root_path = Path::new(&args.path);
        if !root_path.exists() {
            bail!("Path does not exist: {}", args.path);
        }

        // Look up configured name; fall back to model code
        let name = settings.vehicle.iter()
            .find(|v| v.vehicle == args.model && v.year == args.year)
            .map(|v| v.name.clone())
            .unwrap_or_else(|| args.model.clone());

        // Look for a vehicle image in the source path
        let image = ["vehicle.png", "vehicle.jpg", "image.png", "image.jpg"]
            .iter()
            .map(|n| root_path.join(n))
            .find(|p| p.exists())
            .and_then(|p| fs::read(p).ok())
            .unwrap_or_default();
        if image.is_empty() {
            println!("note: no vehicle image found (drop vehicle.png in {} to set one)", args.path);
        }

        self.content_store.upsert_vehicle_direct(&Vehicle {
            id: None,
            year: args.year,
            vehicle: args.model.clone(),
            name,
            image,
        })?;

        println!(
            "Loading Ferrari documents from {} for {} {}",
            args.path, args.year, args.model
        );

        // Create root node for this vehicle
        let root_node_id = self.generate_node_id(&args.model, args.year, "000");
        let root_node = TreeNode {
            node_id: root_node_id,
            vehicle: Some(args.model.clone()),
            year: Some(args.year),
            node_value: "000".to_string(),
            name: Some(args.model.clone()),
            illustration_id: 0,
            location: Some("000".to_string()),
            filter_applies: Some(false),
        };
        self.content_store.upsert_tree_node_direct(&root_node)?;

        // Process directory structure
        self.process_directory(root_path, root_node_id, &args.model, args.year, "000")?;

        println!("Ferrari document loading complete");
        Ok(())
    }

    fn process_directory(
        &self,
        dir_path: &Path,
        parent_node_id: i32,
        model: &str,
        year: i32,
        parent_location: &str,
    ) -> Result<()> {
        let mut entries: Vec<_> = fs::read_dir(dir_path)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.path());

        let mut child_index = 0;
        for entry in entries {
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            if path.is_dir() {
                child_index += 1;
                let location = format!("{}{:03}", parent_location, child_index);
                let node_id = self.generate_node_id(model, year, &location);

                // Parse directory name to extract code and description
                let (code, description) = self.parse_directory_name(&file_name);

                let node = TreeNode {
                    node_id,
                    vehicle: Some(model.to_string()),
                    year: Some(year),
                    node_value: code.clone(),
                    name: Some(description.clone()),
                    illustration_id: 0,
                    location: Some(location.clone()),
                    filter_applies: Some(false),
                };

                self.content_store.upsert_tree_node_direct(&node)?;
                self.content_store.upsert_tree_node_links(parent_node_id, node_id)?;

                println!("  Created node: {} - {} (id: {}, location: {})", code, description, node_id, location);

                // Recursively process subdirectory
                self.process_directory(&path, node_id, model, year, &location)?;
            } else if path.extension().map_or(false, |ext| ext == "pdf") {
                // Process PDF file - create a document node for it
                child_index += 1;
                self.process_pdf(&path, parent_node_id, model, year, parent_location, child_index)?;
            }
        }

        Ok(())
    }

    fn process_pdf(
        &self,
        pdf_path: &Path,
        parent_node_id: i32,
        model: &str,
        year: i32,
        parent_location: &str,
        doc_index: i32,
    ) -> Result<()> {
        let file_name = pdf_path.file_name().unwrap().to_string_lossy().to_string();
        //
        // if file_name != "F2-07 Fuses and relays.pdf" {
        //     return Ok(());
        // }

        // Parse filename to extract code and title separately
        // e.g., "B4-01 Engine air intake system layout.pdf" -> ("B4-01", "Engine air intake system layout")
        let (vehicle_component, title) = self.parse_document_name(&file_name);

        // Generate unique hkap_id for this document using the doc code
        let hkap_id = self.generate_hkap_id(model, year, &vehicle_component);

        // Create a node for this document (like Porsche structure)
        let location = format!("{}{:03}", parent_location, doc_index);
        let doc_node_id = self.generate_node_id(model, year, &location);

        let doc_node = TreeNode {
            node_id: doc_node_id,
            vehicle: Some(model.to_string()),
            year: Some(year),
            node_value: vehicle_component.clone(),
            name: Some(title.clone()),
            illustration_id: 0,
            location: Some(location),
            filter_applies: Some(false),
        };

        self.content_store.upsert_tree_node_direct(&doc_node)?;
        self.content_store.upsert_tree_node_links(parent_node_id, doc_node_id)?;

        // Read PDF content and update links
        let raw_content = fs::read(pdf_path)?;
        let content = self.update_links(&raw_content, year, model)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to update PDF links for {}: {}", file_name, e);
                raw_content
            });

        let media_cloud_file_id = self.generate_consistent_uuid(model, year, file_name.to_string().as_str()).to_string();
        self.content_store.upsert_media_image(&media_cloud_file_id.to_string(), &content)?;

        let workshop_literature = Response {
            payload: WorkshopLiterature {
                file_format: FileFormat::Pdf,
                language_code: LanguageCode::EnUs,
                hkap_id: hkap_id.clone(),
                variant_id: 1,
                version: None,
                latest_version: None,
                version_source_system: None,
                source_system: None,
                kdnr: vehicle_component.clone(),
                ti_number: None,
                publication_date: "2026-01-28T16:35:14.636+02:00".to_string(),
                modification_date: 0,
                title: title.clone(),
                file_name,
                document_type: DocumentType::Mr,
                target_hkap_id: None,
                content: None,
                toc: None,
                techvalues: None,
                mediacloud_image_ids: None,
                tools: None,
                media_cloud_file_id: Some(media_cloud_file_id),
                issue_date: None,
                vehicle_component_with_document_index: None,
                links: None,
                quality_line_segment: None,
                parts: None,
                laborpos: None,
                raw_content: None,
            },
            links: None,
        };

        let document = Document {
            hkap_id: workshop_literature.payload.hkap_id.to_string(),
            variant_id: workshop_literature.payload.variant_id.to_string(),
            language_code: "en-us".to_string(),
            version: 1,
            vehicle_component: vehicle_component.clone(),
            title: title.clone(),
            document_type: "MR".to_string(),
            publication_date: workshop_literature.payload.publication_date.to_string(),
            file_format: "pdf".to_string(),
            vehicle_component_with_document_index: vehicle_component.to_string(),
            new: false,
            bookmarked: false,
            content: serde_json::to_vec(&json!(workshop_literature))?,
        };

        self.content_store.upsert_document_direct(&document)?;
        self.content_store.link_document_to_node(doc_node_id, &hkap_id)?;

        println!("    Added document node: {} - {}", doc_node.node_value, title);
        Ok(())
    }

    fn stable_uuid(&self, model: &str, year: i32, key: &str) -> Uuid {
        let name = format!("ferrari-{}-{}-{}", model, year, key);
        Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes())
    }

    fn generate_node_id(&self, model: &str, year: i32, location: &str) -> i32 {
        let bytes = self.stable_uuid(model, year, location).into_bytes();
        let val = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        val & 0x7FFFFFFF
    }

    fn generate_hkap_id(&self, model: &str, year: i32, doc_code: &str) -> String {
        format!("ferrari-{}", self.stable_uuid(model, year, doc_code).simple())
    }

    fn generate_consistent_uuid(&self, model: &str, year: i32, document_name: &str) -> Uuid {
        self.stable_uuid(model, year, document_name)
    }

    fn parse_document_name(&self, filename: &str) -> (String, String) {
        // Parse filename to extract code and title
        // e.g., "B4-01 Engine air intake system layout.pdf" -> ("B4-01", "Engine air intake system layout")
        // e.g., "01-02 Consultation.pdf" -> ("01-02", "Consultation")
        let name = filename.trim_end_matches(".pdf");
        if let Some(space_idx) = name.find(' ') {
            let code = name[..space_idx].trim().to_string();
            let title = name[space_idx + 1..].trim().to_string();
            (code, title)
        } else {
            (name.to_string(), name.to_string())
        }
    }

    fn parse_directory_name(&self, dir_name: &str) -> (String, String) {
        // Parse directory names like:
        // "A - Fluids, Lubricants, Maintenance" → ("A", "Fluids, Lubricants, Maintenance")
        // "A1 Refilling" → ("A1", "Refilling")
        // "0 - General" → ("0", "General")
        // "1 Manual Structure" → ("1", "Manual Structure")

        // Try pattern with " - " separator first (e.g., "A - Description")
        if let Some(idx) = dir_name.find(" - ") {
            let code = dir_name[..idx].trim().to_string();
            let desc = dir_name[idx + 3..].trim().to_string();
            return (code, desc);
        }

        // Otherwise split on first space (e.g., "A1 Description")
        if let Some(space_idx) = dir_name.find(' ') {
            let code = dir_name[..space_idx].trim().to_string();
            let desc = dir_name[space_idx + 1..].trim().to_string();
            return (code, desc);
        }

        // Fallback: use the whole name as both
        (dir_name.to_string(), dir_name.to_string())
    }

    fn update_links(&self, pdf_content: &[u8], year: i32, model: &str) -> Result<Vec<u8>> {
        let mut doc = PdfDocument::load_mem(pdf_content)?;

        // Update link annotations (clickable hyperlinks)
        self.update_link_annotations(&mut doc, year, model)?;

        // Delete headers from document
        self.delete_document_header(&mut doc)?;

        // Save modified PDF to bytes
        let mut output = Cursor::new(Vec::new());
        doc.save_to(&mut output)?;
        Ok(output.into_inner())
    }

    fn delete_document_header(&self, doc: &mut PdfDocument) -> Result<()> {
        // The PDF uses hex-encoded text with font encoding offset of 29 from ASCII
        // Header patterns to remove:
        // - "FERRARI" encodes to: 002900280035003500240035002C
        // - "modiscs.ferrari.it" URL pattern
        // - Date pattern at top of page
        // - Page numbers like "1 / 12"
        let ferrari_hex_lower = "002900280035003500240035002c"; // "FERRARI" lowercase hex

        // Collect object IDs that need modification
        let objects_to_modify: Vec<ObjectId> = doc.objects.iter()
            .filter_map(|(id, obj)| {
                if let Object::Stream(stream) = obj {
                    // Skip image streams
                    if let Ok(Object::Name(subtype)) = stream.dict.get(b"Subtype") {
                        if subtype == b"Image" {
                            return None;
                        }
                    }

                    let mut stream_clone = stream.clone();
                    stream_clone.decompress();
                    let content = String::from_utf8_lossy(&stream_clone.content).to_string();

                    // Check if this stream contains the Ferrari header
                    if content.to_lowercase().contains(ferrari_hex_lower) {
                        return Some(*id);
                    }
                }
                None
            })
            .collect();

        // Now modify each stream to remove the header
        for id in objects_to_modify {
            if let Some(Object::Stream(stream)) = doc.objects.get_mut(&id) {
                // Decompress the stream
                stream.decompress();
                let content = String::from_utf8_lossy(&stream.content).to_string();

                // Remove header BT...ET blocks
                let modified_content = self.remove_header_blocks(&content);

                // Update the stream content
                stream.set_content(modified_content.into_bytes());

                // Re-compress the stream
                let _ = stream.compress();

                // println!("Removed header from stream object {:?}", id);
            }
        }

        // Verify removal - check if any streams still contain the header
        let remaining_headers: Vec<ObjectId> = doc.objects.iter()
            .filter_map(|(id, obj)| {
                if let Object::Stream(stream) = obj {
                    if let Ok(Object::Name(subtype)) = stream.dict.get(b"Subtype") {
                        if subtype == b"Image" {
                            return None;
                        }
                    }
                    let mut stream_clone = stream.clone();
                    stream_clone.decompress();
                    let content = String::from_utf8_lossy(&stream_clone.content).to_string();
                    if content.to_lowercase().contains(ferrari_hex_lower) {
                        return Some(*id);
                    }
                }
                None
            })
            .collect();

        if remaining_headers.is_empty() {
            // println!("Verified: All FERRARI headers successfully removed");
        } else {
            println!("Warning: {} streams still contain header text: {:?}", remaining_headers.len(), remaining_headers);
        }

        Ok(())
    }

    /// Remove BT...ET blocks that contain header text (at top of page, y=22 or y=775)
    fn remove_header_blocks(&self, content: &str) -> String {
        // Header text patterns (hex-encoded with offset 29):
        // - FERRARI: 002900280035003500240035002C
        // - modiscs.ferrari.it URL
        // - Date like "2/1/2021"
        // - Page numbers at bottom

        // The header appears at y-position 22 (top) and footer at y-position 775 (bottom)
        // We'll remove BT...ET blocks that:
        // 1. Contain "FERRARI" hex pattern
        // 2. Contain the modiscs URL hex pattern
        // 3. Are positioned at y=22 or y=775

        let mut result = String::new();
        let mut chars = content.chars().peekable();
        let mut in_bt_block = false;
        let mut current_block = String::new();

        while let Some(ch) = chars.next() {
            if !in_bt_block {
                // Check for "BT" start
                if ch == 'B' && chars.peek() == Some(&'T') {
                    chars.next(); // consume 'T'
                    // Check if next char is whitespace (to avoid matching other "BT" in hex)
                    if chars.peek().map_or(true, |c| c.is_whitespace()) {
                        in_bt_block = true;
                        current_block = "BT".to_string();
                        continue;
                    } else {
                        result.push('B');
                        result.push('T');
                        continue;
                    }
                }
                result.push(ch);
            } else {
                current_block.push(ch);
                // Check for "ET" end
                if current_block.ends_with("ET") {
                    // Check if this block should be removed
                    let block_lower = current_block.to_lowercase();

                    // Check for header content patterns
                    let is_header = block_lower.contains("002900280035003500240035002c") // FERRARI
                        || block_lower.contains("00500052004700480056004600560011004900480055005500440055004c") // modiscs.ferrari
                        || block_lower.contains("004b0057005700530056001d00120012005000520047004c005600460056") // https://modiscs
                        || (block_lower.contains("1 0 0 -1") && block_lower.contains(" 22 tm")); // Top header position

                    // Check for footer (page numbers at bottom)
                    let is_footer = block_lower.contains("1 0 0 -1") && block_lower.contains(" 775 tm");

                    if is_header || is_footer {
                        // Skip this block (don't add to result)
                        // But we need to preserve the newline after ET
                        if chars.peek() == Some(&'\n') {
                            chars.next();
                        }
                    } else {
                        // Keep this block
                        result.push_str(&current_block);
                    }

                    in_bt_block = false;
                    current_block.clear();
                }
            }
        }

        // If we ended while still in a block, add it
        if !current_block.is_empty() {
            result.push_str(&current_block);
        }

        result
    }

    /// Extract document code from Ferrari link parameter
    /// e.g., "link::SM_SM_F201;..." -> "F2-01"
    /// e.g., "link::SM_SM_B308;..." -> "B3-08"
    fn extract_doc_code_from_link(&self, uri: &str) -> Option<String> {
        // Parse URL to get the 'link' query parameter
        // URL format: Main?action=mo&link=link::SM_SM_F201;Ferrari-...
        let link_pattern = Regex::new(r"link=link::SM_SM_([A-Z])(\d)(\d{2});").unwrap();

        if let Some(caps) = link_pattern.captures(uri) {
            let letter = caps.get(1)?.as_str();
            let first_digit = caps.get(2)?.as_str();
            let remaining = caps.get(3)?.as_str();
            // Convert F201 -> F2-01
            Some(format!("{}{}-{}", letter, first_digit, remaining))
        } else {
            None
        }
    }

    /// Build the new URL for a document reference
    fn build_document_url(&self, year: i32, model: &str, doc_code: &str) -> String {
        let hkap_id = self.generate_hkap_id(model, year, doc_code);
        format!("{}/documents/{}/{}/{}", ROOT, year, model, hkap_id)
    }

    fn update_link_annotations(&self, doc: &mut PdfDocument, year: i32, model: &str) -> Result<()> {
        // Collect all object IDs that need to be updated
        let updates: Vec<(ObjectId, String)> = doc.objects.iter()
            .filter_map(|(id, obj)| {
                if let Object::Dictionary(dict) = obj {
                    if let Ok(Object::Dictionary(d)) = dict.get(b"A") {
                        if let Ok(Object::String(uri_bytes, _)) = d.get(b"URI") {
                            let uri = String::from_utf8_lossy(uri_bytes).to_string();
                            if uri.starts_with(FERRARI_LINK_PREFIX) {
                                // Extract document code and build new URL
                                if let Some(doc_code) = self.extract_doc_code_from_link(&uri) {
                                    let new_uri = self.build_document_url(year, model, &doc_code);
                                    // println!("Transforming link: {} -> {} (doc: {})", uri, new_uri, doc_code);
                                    return Some((*id, new_uri));
                                } else {
                                    println!("Could not parse Ferrari link: {}", uri);
                                }
                            } else {
                                println!("Found non-Ferrari annotation link: {}", uri);
                            }
                        }
                    }
                }
                None
            })
            .collect();

        // Apply updates
        for (id, new_uri) in updates {
            if let Some(Object::Dictionary(dict)) = doc.objects.get_mut(&id) {
                // Get the "A" dictionary, clone it, modify it, and set it back
                if let Ok(Object::Dictionary(action_dict)) = dict.get(b"A") {
                    let mut new_action = action_dict.clone();
                    new_action.set("URI", Object::String(new_uri.into_bytes(), lopdf::StringFormat::Literal));
                    dict.set("A", Object::Dictionary(new_action));
                } else {
                    println!("Error getting object A!");
                }
            }
        }

        Ok(())
    }
}
