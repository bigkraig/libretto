use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkshopLiterature {
    pub file_format: FileFormat,
    pub language_code: LanguageCode,
    pub variant_id: i64,
    pub hkap_id: String,
    pub target_hkap_id: Option<String>,
    pub version: Option<i64>,
    pub latest_version: Option<serde_json::Value>,
    pub version_source_system: Option<String>,
    pub source_system: Option<SourceSystem>,
    pub kdnr: String,
    pub ti_number: Option<String>,
    pub publication_date: String,
    pub modification_date: i64,
    pub title: String,
    pub file_name: String,
    pub document_type: DocumentType,
    pub content: Option<Children>,
    pub toc: Option<Vec<Toc>>,
    pub techvalues: Option<Vec<Techvalue>>,
    pub mediacloud_image_ids: Option<MediacloudImageIds>,
    pub tools: Option<Vec<Tool>>,
    pub media_cloud_file_id: Option<String>,
    pub issue_date: Option<String>,
    pub vehicle_component_with_document_index: Option<String>,
    pub links: Option<Vec<Option<serde_json::Value>>>,
    pub quality_line_segment: Option<QualityLineSegment>,
    pub parts: Option<Vec<Part>>,
    pub laborpos: Option<Vec<Laborpo>>,

    #[serde(default, skip_serializing)]
    pub raw_content: Option<Vec<u8>>,
}

impl WorkshopLiterature {
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = Vec::new();
        if let Some(children) = &self.content {
            results.append(&mut children.pick_children(types));
        }

        if let Some(techvalues) = &self.techvalues {
            for tv in techvalues {
                results.append(&mut tv.pick_children(types));
            }
        }

        results
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    // Mostly came from scraping API
    // Part-id specific media
    Mr, // Manufacturer's Certificates - AKA MC
    Sit, // Service Information Technik
    Sy, // Symptom-based workshop manual
    Teq, // Tequipment
    Ti, // Technical Information
    Rm, // Workshop Manual - AKA WM

    // Non part-id specific media
    Cam, // Campaigns
    Diag, // Diagnostic information
    // Em, //Environmental information
    Etm, // Emission test
    Owm, // Owner's Manuals
    Pm, // Paint Manual
    Sf, // Standard forms
    // Sy, // Symptom-based workshop manual
    Tv, // Technical values
    // Wd, // Wiring Diagrams
    West, // Workshop equipment

    // Country specific media
    Td, // PCNA Training Documentation

    // ??
    Faq,
    Gfs,
    TsiCsdCn,
    TsiCsdUs,

    // these came from workshop lit
    #[serde(rename = "")]
    Empty,
    Hbw,
    Rl,
    Tiaktion,
    Tiequip,
    Ubb,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Children {
    NestedList(Vec<Children>),
    ChildList(Vec<ChildItem>),
    ChildItem(Box<ChildItem>),
}

impl Children {
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut result = Vec::new();
        match self {
            Children::NestedList(list) => {
                for children in list {
                    result.append(&mut children.pick_children(types));
                }
            }
            Children::ChildList(child_list) => {
                for children in child_list {
                    result.append(&mut children.pick_children(types));
                }
            }
            Children::ChildItem(item) => {
                result.append(&mut item.pick_children(types));
            }
        }
        result
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ChildItem {
    Content(Content),
    String(String),
    Number(i32),
}

impl ChildItem {
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        match self {
            ChildItem::Content(content) => {
                return content.pick_children(types);
            }
            _ => {}
        }
        vec![]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "inputs")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Content {
    String(String),
    Number(i32),
    Anchor(ContentAnchor),
    Image(ContentImage),
    InvoiceTable(ContentInvoiceTable),
    Laborpos(ContentLaborpos),
    Link(ContentLink),
    List(ContentList),
    Mixed(ContentMixed),
    Nested(ContentNested),
    Paragraph(ContentParagraph),
    Plot(Vec<ContentPlot>),
    Section(ContentSection),
    Static(ContentStatic),
    Table(ContentTable),
    Warning(ContentWarning),

    // Anchor(serde_json::Value),
    // Image(serde_json::Value),
    // InvoiceTable(serde_json::Value),
    // Laborpos(serde_json::Value),
    // Link(serde_json::Value),
    // List(serde_json::Value),
    // Mixed(serde_json::Value),
    // Nested(serde_json::Value),
    // Paragraph(serde_json::Value),
    // Plot(serde_json::Value),
    // Section(serde_json::Value),
    // Static(serde_json::Value),
    // Table(serde_json::Value),
    // Warning(serde_json::Value),
}

impl Content {
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];
        match self {
            Content::Image(content) => return content.pick_children(types),
            Content::InvoiceTable(content) => return content.pick_children(types),
            Content::Link(content) => return content.pick_children(types),
            Content::List(content) => return content.pick_children(types),
            Content::Mixed(content) => return content.pick_children(types),
            Content::Paragraph(content) => return content.pick_children(types),
            Content::Plot(content) => {
                for item in content {
                    results.append(&mut item.children.pick_children(types));
                }
            }
            Content::Section(content) => return content.pick_children(types),
            Content::Table(content) => return content.pick_children(types),
            Content::Warning(content) => return content.pick_children(types),
            _ => if types.contains(&self.content_type()) {
                results.push(self.clone());
            }
        }
        results
    }
}

#[derive(PartialEq)]
pub enum ContentType {
    String,
    Number,
    Anchor,
    Image,
    InvoiceTable,
    Laborpos,
    Link,
    List,
    Mixed,
    Nested,
    Paragraph,
    Plot,
    Section,
    Static,
    Table,
    Warning,
}

impl Content {
    pub fn content_type(&self) -> ContentType {
        match self {
            Content::String(_) => ContentType::String,
            Content::Number(_) => ContentType::Number,
            Content::Anchor(_) => ContentType::Anchor,
            Content::Image(_) => ContentType::Image,
            Content::InvoiceTable(_) => ContentType::InvoiceTable,
            Content::Laborpos(_) => ContentType::Laborpos,
            Content::Link(_) => ContentType::Link,
            Content::List(_) => ContentType::List,
            Content::Mixed(_) => ContentType::Mixed,
            Content::Nested(_) => ContentType::Nested,
            Content::Paragraph(_) => ContentType::Paragraph,
            Content::Plot(_) => ContentType::Plot,
            Content::Section(_) => ContentType::Section,
            Content::Static(_) => ContentType::Static,
            Content::Table(_) => ContentType::Table,
            Content::Warning(_) => ContentType::Warning,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Content::String(_) => None,
            Content::Number(_) => None,
            Content::Anchor(_) => None,
            Content::Image(_) => Some("Image"), // never seen from root scans
            Content::InvoiceTable(_) => None,
            Content::Laborpos(_) => Some("Laborpos"), // never seen from root scans
            Content::Link(_) => Some("Link"), // never seen from root scans
            Content::List(_) => None,
            Content::Mixed(_) => None,
            Content::Nested(_) => None,
            Content::Paragraph(_) => None,
            Content::Plot(_) => None,
            Content::Section(_) => None,
            Content::Static(_) => None,
            Content::Table(_) => None,
            Content::Warning(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentAnchor {
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentImage {
    pub id: String,
    pub mediacloud_small: String,
    pub mediacloud_normal: String,
    pub mediacloud_large: String,
    pub format: String,
    pub key: String,
    pub title: String,
    pub in_table: Option<bool>,
    pub legend: Option<Vec<[LegendElement; 2]>>,
}

impl ContentImage {
    pub fn content_type(&self) -> ContentType {
        ContentType::Image
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        if let Some(legend) = &self.legend {
            for items in legend {
                for item in items {
                    match item {
                        LegendElement::Content(content) => {
                            results.append(&mut content.pick_children(types));
                        }
                        _ => {}
                    }
                }
            }
        }

        if types.contains(&self.content_type()) {
            results.push(Content::Image(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum LegendElement {
    Content(Content),
    String(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentInvoiceTable {
    pub data: Option<Children>,
}

impl ContentInvoiceTable {
    pub fn content_type(&self) -> ContentType {
        ContentType::InvoiceTable
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        if let Some(children) = &self.data {
            results.append(&mut children.pick_children(types));
        }

        if types.contains(&self.content_type()) {
            results.push(Content::InvoiceTable(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentLaborpos {
    pub time_units: String,
    pub title_html: String,
    pub with_html: Option<String>,
    pub without_html: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentList {
    pub items: Children,
}

impl ContentList {
    pub fn content_type(&self) -> ContentType {
        ContentType::List
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        results.append(&mut self.items.pick_children(types));

        if types.contains(&self.content_type()) {
            results.push(Content::List(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentLink {
    pub html: String,
    pub id: Option<String>,
    pub children: Option<Children>,
}

impl ContentLink {
    pub fn content_type(&self) -> ContentType {
        ContentType::Link
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        if let Some(children) = &self.children {
            results.append(&mut children.pick_children(types));
        }

        if types.contains(&self.content_type()) {
            results.push(Content::Link(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentMixed {
    pub children: Children,
}

impl ContentMixed {
    pub fn content_type(&self) -> ContentType {
        ContentType::Mixed
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        results.append(&mut self.children.pick_children(types));

        if types.contains(&self.content_type()) {
            results.push(Content::Mixed(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentNested {
    #[serde(rename = "type")]
    pub nested_type: NestedType,
    pub document_type: Option<DocumentType>,
    pub scenario_id: Option<String>,
    pub apos_number: Option<String>,
    pub hkap_id: Option<String>,
    pub activity: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NestedType {
    Dok,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentParagraph {
    pub children: Children,
}

impl ContentParagraph {
    pub fn content_type(&self) -> ContentType {
        ContentType::Paragraph
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        results.append(&mut self.children.pick_children(types));

        if types.contains(&self.content_type()) {
            results.push(Content::Paragraph(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentPlot {
    pub position: Option<String>,
    pub id: Option<String>,
    pub children: Children,
}

impl ContentPlot {
    pub fn content_type(&self) -> ContentType {
        ContentType::Plot
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        results.append(&mut self.children.pick_children(types));

        if types.contains(&self.content_type()) {
            results.push(Content::Plot(vec![self.clone()])); // TODO this may need to be modified
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentSection {
    pub children: Option<Children>,
    pub title_ui_text: Option<TitleUiText>,
    pub title: Option<String>,
}

impl ContentSection {
    pub fn content_type(&self) -> ContentType {
        ContentType::Section
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        if let Some(children) = &self.children {
            results.append(&mut children.pick_children(types));
        }

        if types.contains(&self.content_type()) {
            results.push(Content::Section(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentStatic {
    pub html: Option<String>,
    pub para: Option<bool>,
    pub translate_ids: Option<TranslateIds>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentTable {
    pub id: Option<String>,
    pub usage_location: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
    pub base_value: Option<String>,
    pub tolerance1: Option<Vec<String>>,
    pub tolerance2: Option<Vec<String>>,
    pub pgwide: Option<bool>,
    pub data: Option<Children>,
    pub header: Option<Children>,
    pub title: Option<String>,
    pub images: Option<Children>,
}

impl ContentTable {
    pub fn content_type(&self) -> ContentType {
        ContentType::Table
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        for root_item in vec![&self.data, &self.header, &self.images] {
            if let Some(children) = root_item {
                results.append(&mut children.pick_children(types));
            }
        }

        if types.contains(&self.content_type()) {
            results.push(Content::Table(self.clone()));
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContentWarning {
    pub category: String,
    pub consequences: Option<Children>,
    pub source: Option<String>,
    pub actions: Option<Children>,
    pub children: Option<Children>,
}

impl ContentWarning {
    pub fn content_type(&self) -> ContentType {
        ContentType::Warning
    }
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        // self.consequences
        for root_item in vec![&self.actions, &self.children, &self.consequences] {
            if let Some(children) = root_item {
                results.append(&mut children.pick_children(types));
            }
        }

        if types.contains(&self.content_type()) {
            results.push(Content::Warning(self.clone()));
        }
        results
    }
}

pub type TranslateIds = Vec<TranslateId>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TranslateId {
    LblCf,
    LblFes5,
    LblSa4,
    LblFrom,
    LblMediaLongPm,
    LblMediaLongWest,
    LblMediaLongOwm,
    LblTo,
    LblAposNumber,
    LblDescription,
    LblInumber,
    LblTechnicalValuesOverviewTableBaseValueCol,
    LblTechnicalValuesOverviewTableDescriptionCol,
    LblTechnicalValuesOverviewTableKindCol,
    LblTechnicalValuesOverviewTableToleranceCol,
    LblTechnicalValuesOverviewTableUsageLocationCol,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TitleUiText {
    LblModelYear,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Pdf,
    Xml,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Laborpo {
    pub label: String,
    pub apos_number: String,
    pub time_units: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LanguageCode {
    #[serde(rename = "en_US")]
    EnUs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediacloudImageIds {
    pub mediacloud_small: Mediacloud,
    pub mediacloud_normal: Mediacloud,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Mediacloud {
    String(String),
    StringArray(Vec<String>),
}

pub struct MediacloudIterator<'a> {
    inner: Vec<&'a String>,
    current: usize,
}

impl<'a> Mediacloud {
    pub fn iter(&'a self) -> MediacloudIterator<'a> {
        let inner = match self {
            Mediacloud::String(s) => {
                if s.is_empty() {
                    Vec::new()
                } else {
                    vec![s]
                }
            }
            Mediacloud::StringArray(arr) => arr.iter().filter(|s| !s.is_empty()).collect(),
        };
        MediacloudIterator { inner, current: 0 }
    }
}

impl<'a> Iterator for MediacloudIterator<'a> {
    type Item = &'a String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.inner.len() {
            let result = self.inner[self.current];
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Part {
    pub part_number: String,
    pub label: String,
    pub extended_label: String,
    pub quantity: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityLineSegment {
    pub location_text: String,
    pub location_type: String,
    pub symptom_text: String,
    pub symptom_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceSystem {
    #[serde(rename = "")]
    Empty,
    Redsys,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Toc {
    pub label: String,
    pub anchor: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: String,
    pub label: String,
    pub kind: ToolKind,
    pub tool_number: String,
    pub tool_display_number: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolKind {
    Ersatzteil,
    #[serde(rename = "handeluebl-wkz")]
    HandelueblWkz,
    Sonderwerkzeug,
    #[serde(rename = "vw-werkzeug")]
    VwWerkzeug,
    Werkstatteinrichtung,
    #[serde(rename = "werkstatteinrichtung-allgemein")]
    WerkstatteinrichtungAllgemein,
    #[serde(rename = "werkstatteinrichtung-fahrzeugspezifisch")]
    WerkstatteinrichtungFahrzeugspezifisch,
    Zubehoer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Techvalue {
    pub id: Option<String>,
    pub usage_location: Option<String>,
    pub description: Option<String>,
    pub kind: Option<String>,
    pub base_value: Option<String>,
    pub tolerance1: Option<Vec<String>>,
    pub tolerance2: Option<Vec<String>>,
    pub images: Option<Children>,
    pub html: Option<String>,
}

impl Techvalue {
    pub fn pick_children(&self, types: &Vec<ContentType>) -> Vec<Content> {
        let mut results = vec![];

        if let Some(children) = &self.images {
            results.append(&mut children.pick_children(types));
        }

        results
    }
}
