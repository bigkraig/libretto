use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Response<T> {
    pub payload: T,
    pub links: Option<Vec<Link>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaIds {
    pub cloud_id: String,
    pub file_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Link {
    pub rel: String,
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TreeNode {
    pub node_value: String,
    pub name: Option<String>,
    pub illustration_id: i32,
    pub location: Option<String>,
    pub filter_applies: bool,
    pub links: Vec<Link>,
}

impl TreeNode {
    pub fn node_id(&self) -> i32 {
        let href = self.links.iter().find(|l| l.rel == "self").map(|l| l.href.clone()).expect("unable to find self in links");
        let id = href.split("/").last().unwrap().parse().unwrap();
        id
    }
    pub fn children_links(&self) -> Vec<String> {
        self.links.iter().filter(|l| l.rel == "children").map(|l| l.href.clone()).collect()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaCloudFile {
    pub cloud_id: String,
    pub file_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolImage {
    pub small_file_url: String,
    pub normal_file_url: String,
    pub large_file_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tool {
    pub title: String,
    pub tool_number_pag: String,
    pub tool_number_vw: Option<String>,
    pub tool_type: String,
    pub tool_distributors: Vec<ToolDistributor>,
    pub dealer_classification: String,
    pub utilising_vehicle_models: Vec<VehicleModel>,
    pub referencing_documents: Vec<ReferencingDocument>,
    pub description: String,
    pub hook_code: Option<String>,
    pub model_series: Option<String>,
    pub order_type: Option<String>,
    pub tool_order_number: Option<String>,
    pub state: String,
    pub links: Vec<Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferencingDocument {
    pub id: i64,
    pub document_type: crate::workshop_literature::DocumentType,
    pub hkap_id: String,
    pub variant_id: String,
    pub language_code: LanguageCode,
    pub file_format: FileFormat,
    pub title: String,
    pub vehicle_component: String,
    pub document_index: Option<String>,
    pub vehicle_models: Vec<VehicleModel>,
    pub tool_utilisations: Vec<ToolUtilisation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum FileFormat {
    Xml,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LanguageCode {
    #[serde(rename = "en_US")]
    EnUs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolUtilisation {
    pub tool_number_pag: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VehicleModel {
    pub id: Option<String>,
    pub model_year: ModelYear,
    pub model_series: String,
    pub order_type: String,
    pub external_publication_date: Option<String>,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelYear {
    pub model_year: i64,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolDistributor {
    pub part_number: String,
    pub distributor_code: String,
    pub name: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTexts {
    pub components: Vec<String>,
    pub locale: String,
    pub last_update: String,
    pub translations: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Part {
    pub part_id: String,
    pub paw_relevant: bool,
    pub text: Option<String>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Document {
    pub hkap_id: String,
    pub variant_id: String,
    pub language_code: String,
    pub version: i32,
    pub version_source_system: Value,
    pub source_system: Value,
    pub vehicle_component: String,
    pub title: String,
    pub document_type: String,
    pub publication_date: String,
    pub modification_date: Value,
    pub file_format: String,
    pub vehicle_component_with_document_index: String,
    pub new: bool,
    pub bookmarked: bool,
    pub links: Vec<Link>,
}
