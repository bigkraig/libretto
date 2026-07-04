use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    http::StatusCode,
    Json,
    Router, routing::get, routing::post,
};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header};
use axum::response::{Html, IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use mime_sniffer::MimeTypeSniffer;
use serde_derive::{Deserialize, Serialize};
use struct_convert::Convert;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tracing::error;
use crate::content_store::ContentStore;
use crate::models::{Document, TreeNode, Vehicle};
use crate::settings::Settings;

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
pub struct ApiArgs {}

pub struct Api {
    content_store: ContentStore,
    settings: Settings,
}

impl Api {
    pub fn new(settings: &Settings, _args: &ApiArgs) -> Self {
        let content_store = ContentStore::new(&settings);

        Api {
            settings: settings.clone(),
            content_store,
        }
    }
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = if self.0.downcast_ref::<crate::content_store::Error>()
            .map(|e| e.is_not_found())
            .unwrap_or(false)
        {
            StatusCode::NOT_FOUND
        } else {
            error!("{:?}", self.0);
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, format!("{}", self.0)).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn serve(bind_address: &String, api: Api) -> Result<(), anyhow::Error> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(api);

    let vehicles_routes = Router::new()
        .route("/", get(get_vehicles))
        .route("/:year/:model", get(get_vehicle))
        .route("/:year/:model/illustrations", get(get_vehicle_illustration))
        .route("/:year/:model/documents/search", get(search_documents_by_vehicle));

    let vehicle_component_tree_routes = Router::new()
        .route("/:year/:model", get(root_tree_node))
        .route("/:year/:model/", get(root_tree_node))
        .route("/:year/:model/documents", get(get_vehicle_documents))
        .route("/nodes/:node_id", get(get_tree_node))
        .route("/nodes/:node_id/documents", get(get_documents))
        .route("/nodes/:node_id/documents/search", get(search_documents_in_subtree))
        .route("/illustrations/:illustration_id", get(get_illustration));

    // need to add:
    // workshop lit

    // media image
    let workshop_literature_routes = Router::new()
        .route("/search", post(search_documents))
        .route("/:hkap_id", get(get_document));

    let content_routes = Router::new()
        .route("/workshop_image/:image_id/:size", get(get_workshop_image))
        .route("/media/:media_id", get(get_media))
        .route("/tool_data/:year/:model/:tool_id", get(get_tool_data))
        .route("/tool_image/:image_id", get(get_tool_image))
        .route("/translations", get(get_translations));

    let app = Router::new()
        .nest("/v1/vehicles", vehicles_routes)
        .nest("/v1/vehicle_component_tree", vehicle_component_tree_routes)
        .nest("/v1/workshop_literature", workshop_literature_routes)
        .nest("/v1/content", content_routes)
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct DocumentsResponse {
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
}

impl From<&Document> for DocumentsResponse {
    fn from(item: &Document) -> Self {
        DocumentsResponse {
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
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SearchDocumentsRequest {
    pub vehicle: String,
    pub year: i32,
    pub apos_number: String,
    pub document_type: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct TreeNodeResponse {
    node_id: i32,
    vehicle: String,
    year: i32,
    node_value: String,
    name: Option<String>,
    illustration_id: i32,
    location: Option<String>,
    filter_applies: Option<bool>,
    is_folder: bool,
    total_children: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_node_id: Option<i32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<TreeNodeResponse>,
}

impl From<&TreeNode> for TreeNodeResponse {
    fn from(item: &TreeNode) -> Self {
        TreeNodeResponse {
            node_id: item.node_id,
            vehicle: item.vehicle.clone().unwrap(),
            year: item.year.unwrap(),
            node_value: item.node_value.clone(),
            name: item.name.clone(),
            illustration_id: item.illustration_id,
            location: item.location.clone(),
            filter_applies: item.filter_applies,
            is_folder: false,
            total_children: 0,
            parent_node_id: None,
            children: vec![],
        }
    }
}

impl From<TreeNode> for TreeNodeResponse {
    fn from(item: TreeNode) -> Self {
        TreeNodeResponse {
            node_id: item.node_id,
            vehicle: item.vehicle.clone().unwrap(),
            year: item.year.unwrap(),
            node_value: item.node_value.clone(),
            name: item.name.clone(),
            illustration_id: item.illustration_id,
            location: item.location.clone(),
            filter_applies: item.filter_applies,
            is_folder: false,
            total_children: 0,
            parent_node_id: None,
            children: vec![],
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct VehicleResponse {
    year: i32,
    vehicle: String,
    name: String,
    image_url: String,
}

impl From<Vehicle> for VehicleResponse {
    fn from(item: Vehicle) -> Self {
        VehicleResponse {
            vehicle: item.vehicle.clone(),
            year: item.year,
            name: item.name.clone(),
            image_url: "".to_string(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct ToolDataResponse {
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub distributors: Vec<crate::models::ToolDistributor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub referencing_documents: Vec<ReferencingDocument>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Convert)]
#[convert(from = "crate::models::Document")]
pub struct ReferencingDocument {
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
}

impl From<crate::models::Tool> for ToolDataResponse {
    fn from(item: crate::models::Tool) -> Self {
        ToolDataResponse {
            title: item.title.clone(),
            tool_number_pag: item.tool_number_pag.clone(),
            tool_number_vw: item.tool_number_vw.clone(),
            tool_type: item.tool_type.clone(),
            dealer_classification: item.dealer_classification.clone(),
            description: item.description.clone(),
            hook_code: item.hook_code.clone(),
            model_series: item.model_series.clone(),
            order_type: item.order_type.clone(),
            tool_order_number: item.tool_order_number.clone(),
            state: item.state.clone(),
            distributors: vec![],
            referencing_documents: vec![],
        }
    }
}

async fn get_vehicles(State(state): State<Arc<Api>>) -> Result<Json<Vec<VehicleResponse>>, AppError> {
    let vehicles = state.content_store.get_vehicles()?.into_iter().map(|vehicle| {
        let mut new: VehicleResponse = vehicle.into();
        new.image_url = format!("{}/v1/vehicles/{}/{}/illustrations", state.settings.api.host, new.year, &new.vehicle);
        new
    }).collect();
    Ok(Json(vehicles))
}

async fn get_vehicle(State(state): State<Arc<Api>>, Path((year, vehicle)): Path<(i32, String)>) -> Result<Json<VehicleResponse>, AppError> {
    let vehicle = state.content_store.get_vehicle(year, &vehicle)?;
    let mut new: VehicleResponse = vehicle.into();
    new.image_url = format!("{}/v1/vehicles/{}/{}/illustrations", state.settings.api.host, new.year, &new.vehicle);
    Ok(Json(new))
}

async fn get_vehicle_illustration(State(state): State<Arc<Api>>, Path((year, vehicle)): Path<(i32, String)>) -> impl IntoResponse {
    let illustration = match state.content_store.get_vehicle_illustration(year, &vehicle) {
        Ok(ill) => ill,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string()))
    };

    let content_type = match mime_guess::from_ext("png").first_raw() {
        Some(mime) => mime,
        None => return Err((StatusCode::BAD_REQUEST, "MIME Type couldn't be determined".to_string()))
    };

    let headers = [
        (header::CONTENT_TYPE, content_type),
    ];

    let body = Body::from(illustration);
    Ok((headers, body))
}

fn get_children_nodes(state: Arc<Api>, node_id: i32) -> Result<Vec<TreeNodeResponse>, AppError> {
    let children: Vec<TreeNodeResponse> = state.content_store.get_children_nodes(node_id)?.iter().filter_map(|c| {
        let mut r: TreeNodeResponse = c.into();
        if let Ok(links) = state.content_store.get_links(r.node_id) {
            r.is_folder = links.len() > 0;
            r.total_children = links.len();
        }
        // Skip nodes with no documents anywhere in their subtree
        match state.content_store.subtree_has_documents(r.node_id) {
            Ok(true) => Some(r),
            _ => None,
        }
    }).collect();
    Ok(children)
}

async fn get_translations(State(state): State<Arc<Api>>) -> Result<Json<HashMap<String, String>>, AppError> {
    let translations = state.content_store.get_translations()?;
    Ok(Json(translations))
}


async fn root_tree_node(State(state): State<Arc<Api>>, Path((year, vehicle)): Path<(i32, String)>) -> Result<Json<TreeNodeResponse>, AppError> {
    let node = state.content_store.get_tree_node(&vehicle, year, None)?;
    let children = get_children_nodes(state, node.node_id)?;
    let mut response: TreeNodeResponse = node.into();
    response.is_folder = children.len() > 0;
    response.total_children = children.len();
    response.children = children;
    Ok(Json(response))
}

async fn get_tree_node(State(state): State<Arc<Api>>, Path(node_id): Path<i32>) -> Result<Json<TreeNodeResponse>, AppError> {
    let node = state.content_store.get_tree_node_by_id(node_id)?;

    let mut result: TreeNodeResponse = node.clone().into();

    if let Ok(parent_id) = state.content_store.get_parent(node.node_id) {
        result.parent_node_id = Some(parent_id);
    }

    result.children = get_children_nodes(state, node.node_id)?;
    result.is_folder = result.children.len() > 0;
    result.total_children = result.children.len();
    Ok(Json(result))
}


async fn get_illustration(State(state): State<Arc<Api>>, Path(illustration_id): Path<i32>) -> Result<Html<String>, AppError> {
    let content = state.content_store.get_illustration(illustration_id)?;
    Ok(Html(content))
}

async fn get_tool_data(State(state): State<Arc<Api>>, Path((year, vehicle, tool_id)): Path<(i32, String, String)>) -> Result<Json<ToolDataResponse>, AppError> {
    println!("Getting tool: {}", tool_id);
    let decoded = STANDARD.decode(tool_id.as_bytes()).expect("Failed to decode image id");
    let tool_id = &String::from_utf8(decoded.clone()).expect("Failed to convert tool id to string");
    let tool = state.content_store.get_tool_data(tool_id)?;
    let mut result: ToolDataResponse = tool.clone().into();
    result.distributors = state.content_store.get_tool_distributors(tool.id.unwrap())?.into();
    result.referencing_documents = state.content_store.get_referencing_documents(year, vehicle, tool.id.unwrap())?.iter().map(|d| d.clone().into()).collect();
    Ok(Json(result))
}

async fn get_tool_image(State(state): State<Arc<Api>>, Path(image_id): Path<String>) -> Response {
    let decoded = STANDARD.decode(image_id.as_bytes()).expect("Failed to decode image id");
    let content = match state.content_store.get_tool_image(&String::from_utf8(decoded).expect("Failed to convert image id to string")) {
        Ok(x) => x,
        Err(e) => return (StatusCode::NOT_FOUND, format!("Tool image not found: {}", e).to_string()).into_response()
    };

    let content_type = content.sniff_mime_type().unwrap_or_else(|| "image/svg+xml");

    (StatusCode::OK, [("content-type", content_type)], content.clone()).into_response()
}

async fn get_media(State(state): State<Arc<Api>>, Path(media_id): Path<String>) -> Response {
    let content = match state.content_store.get_media(&media_id) {
        Ok(x) => x,
        Err(e) => return (StatusCode::NOT_FOUND, format!("Media id not found: {}", e).to_string()).into_response()
    };

    let content_type = content.sniff_mime_type().unwrap_or_else(|| "image/svg+xml");

    (StatusCode::OK, [("content-type", content_type)], content.clone()).into_response()
}

async fn get_workshop_image(State(state): State<Arc<Api>>, Path((image_id, size)): Path<(i32, String)>) -> Response {
    let content = match state.content_store.get_workshop_image(image_id, &size) {
        Ok(x) => x,
        Err(e) => return (StatusCode::NOT_FOUND, format!("Workshop image not found: {}", e).to_string()).into_response()
    };

    let content_type = content.sniff_mime_type().unwrap_or_else(|| "image/svg+xml");

    (StatusCode::OK, [("content-type", content_type)], content.clone()).into_response()
}

async fn get_document(State(state): State<Arc<Api>>, Path(hkap_id): Path<String>) -> Result<String, AppError> {
    let result = state.content_store.get_document(&hkap_id)?;
    println!("Getting document: {} - {} {} {:?}", hkap_id, result.document_type, result.title, result.vehicle_component_with_document_index);
    let s = jsonxf::pretty_print(String::from_utf8(result.content.clone())?.as_str()).unwrap();
    Ok(s.into())
}

async fn search_documents(State(state): State<Arc<Api>>, body: Json<SearchDocumentsRequest>) -> Result<Json<Vec<DocumentsResponse>>, AppError> {
    let result = state.content_store.search_documents(&body.vehicle, body.year, &body.document_type, &body.apos_number)?;
    Ok(Json(result.iter().map(|d| d.into()).collect::<Vec<DocumentsResponse>>()))
}

async fn get_documents(State(state): State<Arc<Api>>, Path(node_id): Path<i32>) -> Result<Json<Vec<DocumentsResponse>>, AppError> {
    let result = state.content_store.list_documents_by_node_id(node_id)?.iter().map(|d| d.into()).collect::<Vec<DocumentsResponse>>();
    Ok(Json(result))
}

// Every document for a vehicle (the root node's whole subtree), so the list can
// show data the moment a vehicle is picked, before drilling into a component.
async fn get_vehicle_documents(State(state): State<Arc<Api>>, Path((year, vehicle)): Path<(i32, String)>) -> Result<Json<Vec<DocumentsResponse>>, AppError> {
    let root = state.content_store.get_tree_node(&vehicle, year, None)?;
    let result = state.content_store.list_documents_by_node_id(root.node_id)?.iter().map(|d| d.into()).collect::<Vec<DocumentsResponse>>();
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
struct SubtreeSearchQuery {
    q: String,
}

async fn search_documents_in_subtree(
    State(state): State<Arc<Api>>,
    Path(node_id): Path<i32>,
    Query(params): Query<SubtreeSearchQuery>,
) -> Result<Json<Vec<DocumentsResponse>>, AppError> {
    let result = state
        .content_store
        .search_documents_in_subtree(node_id, &params.q)?
        .iter()
        .map(|d| d.into())
        .collect::<Vec<DocumentsResponse>>();
    Ok(Json(result))
}

async fn search_documents_by_vehicle(
    State(state): State<Arc<Api>>,
    Path((year, vehicle)): Path<(i32, String)>,
    Query(params): Query<SubtreeSearchQuery>,
) -> Result<Json<Vec<DocumentsResponse>>, AppError> {
    let result = state
        .content_store
        .search_documents_by_vehicle(&vehicle, year, &params.q)?
        .iter()
        .map(|d| d.into())
        .collect::<Vec<DocumentsResponse>>();
    Ok(Json(result))
}
