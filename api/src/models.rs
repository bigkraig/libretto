use std::cmp::Ordering;
use struct_convert::Convert;
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct TreeNode {
    pub node_id: i32,
    pub vehicle: Option<String>,
    pub year: Option<i32>,
    pub node_value: String,
    pub name: Option<String>,
    pub illustration_id: i32,
    pub location: Option<String>,
    pub filter_applies: Option<bool>,
}

impl From<&pcss::api_types::TreeNode> for TreeNode {
    fn from(item: &pcss::api_types::TreeNode) -> Self {
        TreeNode {
            node_id: item.node_id(),
            vehicle: None,
            year: None,
            node_value: item.node_value.clone(),
            name: item.name.clone(),
            illustration_id: item.illustration_id,
            location: item.location.clone(),
            filter_applies: Some(item.filter_applies),
        }
    }
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct TreeNodeLinks {
    pub id: Option<i32>,
    pub parent_node_id: i32,
    pub child_node_id: i32,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct DocumentLinks {
    pub id: Option<i32>,
    pub node_id: i32,
    pub hkap_id: String,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Document {
    pub hkap_id: String,
    pub variant_id: String,
    pub language_code: String,
    pub version: i32,
    pub vehicle_component: String,
    pub title: String,
    pub document_type: String,
    pub publication_date: String,
    pub file_format: String,
    pub vehicle_component_with_document_index: String,
    pub new: bool,
    pub bookmarked: bool,
    pub content: Vec<u8>,
}

fn atoi<F: std::str::FromStr>(input: &str) -> Result<F, <F as std::str::FromStr>::Err> {
    nom::character::complete::digit0::<_, nom::error::Error<_>>(input).unwrap().1.parse::<F>()
}

impl PartialOrd<Self> for Document {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Document {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let s = atoi::<usize>(&self.vehicle_component_with_document_index).unwrap_or(0);
        let o = atoi::<usize>(&other.vehicle_component_with_document_index).unwrap_or(0);

        if s != 0 || o != 0 {
            match s.cmp(&o) {
                Ordering::Equal => {
                    return self.vehicle_component_with_document_index.cmp(&other.vehicle_component_with_document_index);
                }
                ordering => return ordering,
            }
        }

        self.vehicle_component.cmp(&other.vehicle_component)
    }
}

impl From<&pcss::api_types::Document> for Document {
    fn from(item: &pcss::api_types::Document) -> Self {
        Document {
            hkap_id: item.hkap_id.clone(),
            variant_id: item.variant_id.clone(),
            language_code: item.language_code.clone(),
            version: item.version,
            vehicle_component: item.vehicle_component.clone(),
            title: item.title.clone(),
            document_type: item.document_type.clone(),
            publication_date: item.publication_date.clone(),
            file_format: item.file_format.clone(),
            vehicle_component_with_document_index: item.vehicle_component_with_document_index.clone(),
            new: item.new,
            bookmarked: item.bookmarked,
            content: Vec::new(),
        }
    }
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Illustration {
    pub illustration_id: i32,
    pub content: String,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Part {
    pub id: Option<i32>,
    pub vehicle: Option<String>,
    pub year: Option<i32>,
    pub vehicle_component: Option<String>,
    pub part_id: String,
    pub paw_relevant: bool,
    pub text: Option<String>,
}

impl From<&pcss::api_types::Part> for Part {
    fn from(item: &pcss::api_types::Part) -> Self {
        Part {
            id: None,
            vehicle: None,
            year: None,
            part_id: item.part_id.clone(),
            paw_relevant: item.paw_relevant,
            text: item.text.clone(),
            vehicle_component: None,
        }
    }
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    pub id: Option<i32>,
    pub year: i32,
    pub vehicle: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub image: Vec<u8>,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct WorkshopImage {
    pub id: i32,
    pub size: String,
    pub content: Vec<u8>,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct MediaImage {
    pub id: String,
    pub content: Vec<u8>,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct DocumentText {
    pub hkap_id: String,
    pub text: String,
    pub text_squished: String,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct ToolImage {
    pub id: String,
    pub content: Vec<u8>,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Translations {
    pub key: String,
    pub value: String,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone, Convert)]
#[convert(from = "pcss::api_types::Tool")]
pub struct Tool {
    #[serde(skip_serializing)]
    #[convert_field(from = "pcss::api_types::Tool", ignore)]
    pub id: Option<i32>,
    pub title: String,
    pub tool_number_pag: String,
    pub tool_number_vw: Option<String>,
    pub tool_type: String,
    pub dealer_classification: String,
    pub description: String,
    pub hook_code: Option<String>,
    pub model_series: Option<String>,
    pub order_type: Option<String>,
    pub tool_order_number: Option<String>,
    pub state: String,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone, Convert)]
#[convert(from = "pcss::api_types::ToolDistributor")]
pub struct ToolDistributor {
    #[serde(skip_serializing)]
    #[convert_field(from = "pcss::api_types::ToolDistributor", ignore)]
    pub id: Option<i32>,
    pub name: String,
    pub part_number: String,
    pub distributor_code: String,
    pub city: Option<String>,
    pub zip: Option<String>,
    pub street: Option<String>,
    pub phone: Option<String>,
    pub fax: Option<String>,
    pub email: Option<String>,
    pub web: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub key: String,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct ToolDistributorsLink {
    pub id: Option<i32>,
    pub tool_id: i32,
    pub tool_distributor_id: i32,
}

#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct ReferencingToolDocument {
    pub id: Option<i32>,
    pub tool_id: i32,
    pub hkap_id: String,
}

// Lightweight node projection for the ancestor-path (breadcrumb) endpoint.
#[derive(Default, Debug, sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct TreeNodePath {
    pub node_id: i32,
    pub node_value: String,
    pub name: Option<String>,
}
