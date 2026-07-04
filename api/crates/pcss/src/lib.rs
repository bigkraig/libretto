use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hex::encode as hex_encode;
use serde::de::DeserializeOwned;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::api_types::{Document, MediaCloudFile, MediaIds, Part, Response, Tool, ToolImage, TreeNode, UiTexts};
use crate::workshop_literature::{DocumentType, WorkshopLiterature};

const PPN: &str = "https://ppn.porsche.com/pcss";
const MEDIA_IDS_URL: &str = "https://ppn.porsche.com/pcss/workshop_literature/v1/mediaIds";

pub mod api_types;
pub mod workshop_literature;
mod macros;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid illustration")]
    InvalidIllustration,

    #[error("http error {0}")]
    HTTPError(reqwest::StatusCode),

    #[error("invalid json")]
    InvalidJSON,

    #[error("too many roots received")]
    TooManyRoots,
}

pub struct PCSS {
    cookie: String,
    client: reqwest::blocking::Client,
    cache_dir: PathBuf,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct VehicleRequest {
    model_year: i32,
    order_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct DocumentRequest {
    document_type: Vec<DocumentType>,
    search_term: Option<()>,
    vehicle_component: String,
    vehicle_request: VehicleRequest,
}

fn cache_key(input: &str) -> String {
    hex_encode(Sha256::digest(input.as_bytes()))
}

impl PCSS {
    pub fn new(cache_dir: &str, cookie: &str) -> Self {
        let dir = PathBuf::from(cache_dir);
        fs::create_dir_all(dir.join("web")).expect("could not create cache/web dir");
        fs::create_dir_all(dir.join("media")).expect("could not create cache/media dir");
        PCSS {
            client: reqwest::blocking::Client::new(),
            cache_dir: dir,
            cookie: cookie.to_string(),
        }
    }

    fn web_cache_path(&self, key: &str) -> PathBuf {
        self.cache_dir.join("web").join(key)
    }

    fn media_cache_path(&self, id: &str) -> PathBuf {
        self.cache_dir.join("media").join(id)
    }

    fn add_headers(&self, request: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
        request
            .header("Cookie", &self.cookie)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, image/*, text/plain, */*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Sec-Fetch-Mode", "cors")
            .header("Origin", "https://pcss.porsche.com")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15")
            .header("Referer", "https://pcss.porsche.com")
            .header("Sec-Fetch-Dest", "empty")
    }

    fn raw_get(&self, req_url: String) -> Result<Vec<u8>> {
        // Strip volatile query params from CDN URLs before caching
        let cache_url = if req_url.starts_with("https://cdn.mediacloud.porsche.com/") {
            req_url.split('?').next().unwrap_or(&req_url).to_string()
        } else {
            req_url.clone()
        };

        let path = self.web_cache_path(&cache_key(&cache_url));
        if path.exists() {
            return Ok(fs::read(&path)?);
        }

        println!("GET {}", req_url);
        let mut req = self.client.get(&req_url);
        req = self.add_headers(req);
        let res = req.send()?;

        if res.status() == reqwest::StatusCode::OK {
            let body = res.bytes()?.to_vec();
            fs::write(&path, &body)?;
            return Ok(body);
        }

        bail!(Error::HTTPError(res.status()))
    }

    fn get_json<T: DeserializeOwned>(&self, req_url: String) -> Result<T> {
        let res = self.raw_get(req_url).context("Failed getting URL")?;
        let content = String::from_utf8(res).context("Invalid UTF-8")?;
        let jd = &mut serde_json::Deserializer::from_str(&content);
        let deserialized: Response<T> = serde_path_to_error::deserialize(jd)
            .context("Failed deserializing body")?;
        Ok(deserialized.payload)
    }

    fn get_bytes(&self, req_url: String) -> Result<Vec<u8>> {
        self.raw_get(req_url).context("Failed getting URL")
    }

    fn raw_post(&self, req_url: String, body: Option<String>) -> Result<Option<String>> {
        let cache_input = format!("{}\n{}", req_url, body.as_deref().unwrap_or(""));
        let path = self.web_cache_path(&cache_key(&cache_input));

        if path.exists() {
            return Ok(Some(String::from_utf8(fs::read(&path)?)?));
        }

        println!("POST {}", req_url);
        let mut req = self.client.post(&req_url);
        req = self.add_headers(req);
        if let Some(ref b) = body {
            req = req.body(b.clone());
        }
        let res = req.send()?;

        if res.status() == reqwest::StatusCode::OK {
            let bytes = res.bytes()?.to_vec();
            fs::write(&path, &bytes)?;
            return Ok(Some(String::from_utf8(bytes)?));
        }

        bail!(Error::HTTPError(res.status()))
    }

    fn post<T: DeserializeOwned>(&self, req_url: String, body: Option<String>) -> Result<Response<T>> {
        let res = self.raw_post(req_url, body).context("Failed posting")?;
        if let Some(body) = &res {
            let jd = &mut serde_json::Deserializer::from_str(body);
            let deserialized: Response<T> = serde_path_to_error::deserialize(jd)
                .context("Failed deserializing body")?;
            return Ok(deserialized);
        }
        bail!(Error::InvalidJSON)
    }

    pub fn get_tree_root(&self, vehicle: &String, year: i32) -> Result<Vec<TreeNode>> {
        let url = format!(
            "{}/vehicle_component_tree/v1/en_US/{}/{}/tree-nodes/?roots=true&filter=WORKSHOP_LITERATURE&filter=LABOR_OPERATION",
            PPN, year, vehicle
        );
        let r: Vec<TreeNode> = self.get_json(url)?;
        if r.len() != 1 {
            bail!(Error::TooManyRoots)
        }
        Ok(r)
    }

    pub fn get_children_ids(&self, node: &TreeNode) -> Result<Vec<i32>> {
        let mut nodes: Vec<i32> = vec![];
        for href in node.children_links() {
            let children: Vec<TreeNode> = self.get_json(href)?;
            for child in children {
                nodes.push(child.node_id());
            }
        }
        Ok(nodes)
    }

    fn collect_children(&self, node: &TreeNode) -> Result<Vec<TreeNode>> {
        let mut nodes: Vec<TreeNode> = vec![];
        for mut href in node.children_links() {
            if !href.contains("&filter=WORKSHOP_LITERATURE&filter=LABOR_OPERATION") {
                href.push_str("&filter=WORKSHOP_LITERATURE&filter=LABOR_OPERATION");
            }
            let children: Vec<TreeNode> = self.get_json(href)?;
            for child in children {
                nodes.push(child.clone());
                for c in self.collect_children(&child)? {
                    nodes.push(c);
                }
            }
        }
        Ok(nodes)
    }

    pub fn get_tree_nodes(&self, vehicle: &String, year: i32) -> Result<Vec<TreeNode>> {
        let root_nodes = self.get_tree_root(vehicle, year)?;
        let mut nodes: Vec<TreeNode> = vec![];
        for node in &root_nodes {
            nodes.push(node.clone());
            nodes.append(&mut self.collect_children(node)?);
        }
        nodes.sort();
        nodes.dedup();
        Ok(nodes)
    }

    pub fn get_parts(&self, vehicle: &String, year: i32, vehicle_component: &String) -> Result<Vec<Part>> {
        let content: Vec<Part> = self.get_json(format!(
            "{}/vehicle_component_tree/v1/en_US/{}/{}/partids?validfordc=&filter%5Bvc%5D={}&limit=100&sort%5Bby%5D=partId",
            PPN, year, vehicle, vehicle_component
        ))?;
        Ok(content)
    }

    pub fn get_illustration(&self, illustration_id: i32) -> Result<String> {
        let content = self.get_bytes(format!(
            "https://ppn.porsche.com/pcss/vehicle_component_tree/v1/illustration/{}",
            illustration_id
        ))?;
        Ok(String::from_utf8(content)?)
    }

    pub fn get_ui_texts(&self) -> Result<Vec<UiTexts>> {
        let vehicle = self.get_json("https://pcss.porsche.com/frontend_support/v2/en_US/uitexts?application=PCSS_ALL,SERVICE_VEHICLE".to_string())?;
        let infomedia = self.get_json("https://pcss.porsche.com/frontend_support/v2/en_GB/uitexts?application=PCSS_ALL,SERVICE_INFOMEDIA".to_string())?;
        Ok(vec![vehicle, infomedia])
    }

    pub fn list_workshop_literature(&self, vehicle: &String, year: i32, component: &String) -> Result<Vec<Document>> {
        let vehicle_request = DocumentRequest {
            document_type: vec![
                DocumentType::Ti, DocumentType::Mr, DocumentType::Rm,
                DocumentType::Sit, DocumentType::Sy, DocumentType::Teq,
            ],
            search_term: None,
            vehicle_component: component.clone(),
            vehicle_request: VehicleRequest { order_type: vehicle.clone(), model_year: year },
        };
        let url = "https://ppn.porsche.com/pcss/workshop_literature/v1/en_US/documents?offset=0&limit=100&sort%5Bby%5D=vehicleComponent&sort%5Border%5D=asc".to_string();
        let mut response: Response<Vec<Document>> = self.post(url, Some(json!(vehicle_request).to_string()))?;
        Ok(response.payload)
    }

    pub fn get_workshop_literature(&self, vehicle: &String, year: i32, document: &Document) -> Result<WorkshopLiterature> {
        let url = format!(
            "https://ppn.porsche.com/pcss/workshop_literature/v1/en_US/documents/{}/{}/1?filter%5BorderType%5D={}&filter%5BmodelYear%5D={}&filter%5BengineType%5D=&filter%5BgearType%5D=&version=",
            document.document_type, document.hkap_id, vehicle, year
        );
        let ctx = format!("Check with export_json.py for {}: {} {} {} {}",
            document.hkap_id, vehicle, year, document.document_type, document.hkap_id);
        let raw = self.get_bytes(url).context(ctx.clone())?;
        let content = String::from_utf8(raw.clone()).context("Invalid UTF-8")?;
        let jd = &mut serde_json::Deserializer::from_str(&content);
        let mut wl: Response<WorkshopLiterature> = serde_path_to_error::deserialize(jd).context(ctx)?;
        wl.payload.raw_content = Some(raw);
        Ok(wl.payload)
    }

    pub fn get_media_ids(&self, mut ids: Vec<&String>) -> Result<Vec<(String, Vec<u8>)>> {
        ids.sort();
        ids.dedup();
        let mut result: Vec<(String, Vec<u8>)> = vec![];
        let mut missing: Vec<&String> = vec![];

        for id in &ids {
            let path = self.media_cache_path(id);
            if path.exists() {
                result.push((id.to_string(), fs::read(&path)?));
            } else {
                missing.push(id);
            }
        }

        if missing.is_empty() {
            return Ok(result);
        }

        let content: Response<Vec<MediaIds>> = self.post(
            MEDIA_IDS_URL.to_string(),
            Some(json!(missing).to_string()),
        )?;
        for media in content.payload {
            let bytes = self.get_bytes(media.file_url)?;
            fs::write(self.media_cache_path(&media.cloud_id), &bytes)?;
            result.push((media.cloud_id, bytes));
        }

        Ok(result)
    }

    pub fn get_workshop_image(&self, id: &String, size: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://ppn.porsche.com/pcss/workshop_literature/v1/images/{}?size={}&version=",
            id, size
        );
        self.get_bytes(url)
    }

    pub fn get_tool(&self, id: &str) -> Result<Vec<u8>> {
        let encoded = STANDARD.encode(id.as_bytes());
        let url = format!(
            "https://ppn.porsche.com/pcss/workshop_literature/v2/en_US/toolimage/{}",
            encoded
        );
        if matches!(url.as_str(),
            "https://ppn.porsche.com/pcss/workshop_literature/v2/en_US/toolimage/UDkwMDE5"
            | "https://ppn.porsche.com/pcss/workshop_literature/v2/en_US/toolimage/TnIuMTY4"
            | "https://ppn.porsche.com/pcss/workshop_literature/v2/en_US/toolimage/MTEzOQ=="
        ) {
            return Ok(vec![]);
        }
        let tool: ToolImage = self.get_json(url)?;
        self.get_bytes(tool.large_file_url)
    }

    pub fn get_tool_data(&self, id: &str) -> Result<Tool> {
        let encoded = STANDARD.encode(id.as_bytes());
        let url = format!("https://ppn.porsche.com/pcss/workshop_literature/v2/en_US/tool/{}", encoded);
        self.get_json(url)
    }

    pub fn get_pdf(&self, id: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://ppn.porsche.com/pcss/workshop_literature/v1/mediaId/{}?openFile=true",
            id
        );
        let media: MediaCloudFile = self.get_json(url)?;
        self.get_bytes(media.file_url)
    }
}
