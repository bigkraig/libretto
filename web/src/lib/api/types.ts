export interface IDocument {
  id: number
  hkap_id: string
  // vehicle: string
  // year: string
  variant_id: string
  language_code: string
  version: number
  vehicle_component: string
  title: string
  document_type: string
  publication_date: string
  file_format: string
  vehicle_component_with_document_index: string
  new: boolean
  bookmarked: boolean
}

export class Document {
  id: number;
  hkap_id: string;
  variant_id: string;
  language_code: string;
  version: number;
  vehicle_component: string;
  title: string;
  document_type: string;
  publication_date: string;
  file_format: string;
  vehicle_component_with_document_index: string;
  new: boolean;
  bookmarked: boolean;

  constructor(config: IDocument) {
    this.id = config.id;
    this.hkap_id = config.hkap_id;
    this.variant_id = config.variant_id;
    this.language_code = config.language_code;
    this.version = config.version;
    this.vehicle_component = config.vehicle_component;
    this.title = config.title;
    this.document_type = config.document_type;
    this.publication_date = config.publication_date;
    this.file_format = config.file_format;
    this.vehicle_component_with_document_index = config.vehicle_component_with_document_index;
    this.new = config.new;
    this.bookmarked = config.bookmarked;
  }
}

export interface IVehicle {
  year: number
  name: string
  vehicle: string
  image_url: string
}

export class Vehicle {
  year: number;
  name: string;
  vehicle: string;
  image_url: string;

  constructor(config: IVehicle) {
    this.year = config.year;
    this.name = config.name;
    this.vehicle = config.vehicle;
    this.image_url = config.image_url;
  }
}

export interface ITreeNode {
  node_id: number
  vehicle: string
  year: number
  node_value: string
  name: string
  illustration_id: number
  location: string
  filter_applies: false
  parent_node_id: number | null
  total_children: number;
  children: ITreeNode[]

  isDriveFile(): boolean
}

export class TreeNode {
  node_id: number;
  vehicle: string;
  year: number;
  node_value: string;
  name: string;
  illustration_id: number;
  location: string;
  filter_applies: false;
  parent_node_id: number | null;
  total_children: number;
  children: ITreeNode[];

  constructor(config: ITreeNode) {
    this.node_id = config.node_id;
    this.vehicle = config.vehicle;
    this.year = config.year;
    this.node_value = config.node_value;
    this.name = config && config.name;
    this.illustration_id = config.illustration_id;
    this.location = config.location;
    this.filter_applies = config.filter_applies;
    this.total_children = config.total_children;
    this.parent_node_id = config.parent_node_id;
    this.children = config.children ? config.children.map((c) => new TreeNode(c)) : [];
  }

  isDriveFile(): boolean {
    return (this.total_children == 0)
  }
}

export interface IWorkshopLiterature {
  fileFormat: FileFormat;
  languageCode: LanguageCode;
  variantId: number;
  hkapId: number;
  targetHkapId?: string;
  version?: number;
  latestVersion?: any;
  versionSourceSystem?: string;
  sourceSystem?: SourceSystem;
  kdnr: string;
  tiNumber?: string;
  publicationDate: string;
  modificationDate: number;
  title: string;
  fileName: string;
  documentType: DocumentType;
  content?: Children;
  toc?: Toc[];
  techvalues?: Techvalue[];
  mediacloudImageIds?: MediacloudImageIds;
  tools?: Tool[];
  mediaCloudFileId?: string;
  issueDate?: string;
  vehicleComponentWithDocumentIndex?: string;
  links?: any[];
  qualityLineSegment?: QualityLineSegment;
  parts?: Part[];
  laborpos?: Laborpo[];
}

export type Children = Children[] | Content[] | Content;

export function instanceOfContentAnchor(object: any): object is ContentAnchor {
  return object.type === 'anchor';
}

export function instanceOfContentImage(object: any): object is ContentImage {
  return object.type === 'image';
}

export function instanceOfContentLink(object: any): object is ContentLink {
  return object.type === 'link';
}

export function instanceOfContentList(object: any): object is ContentList {
  return object.type === 'list';
}

export function instanceOfContentMixed(object: any): object is ContentMixed {
  return object.type === 'mixed';
}

export function instanceOfContentNested(object: any): object is ContentNested {
  return object.type === 'nested';
}

export function instanceOfContentParagraph(object: any): object is ContentParagraph {
  return object.type === 'paragraph';
}

export function instanceOfContentPlot(object: any): object is ContentPlot {
  return object.type === 'plot';
}

export function instanceOfContentSection(object: any): object is ContentSection {
  return object.type === 'section';
}

export function instanceOfContentStatic(object: any): object is ContentStatic {
  return object.type === 'static';
}

export function instanceOfContentTable(object: any): object is ContentTable {
  return object.type === 'table';
}

export function instanceOfContentWarning(object: any): object is ContentWarning {
  return object.type === 'warning';
}

export type Content = string | number
  | { type: 'anchor'; value: ContentAnchor }
  | { type: 'image'; value: ContentImage }
  | { type: 'InvoiceTable'; value: ContentInvoiceTable }
  | { type: 'laborpos'; value: ContentLaborpos }
  | { type: 'link'; value: ContentLink }
  | { type: 'list'; value: ContentList }
  | { type: 'mixed'; value: ContentMixed }
  | { type: 'nested'; value: ContentNested }
  | { type: 'paragraph'; value: ContentParagraph }
  | { type: 'plot'; value: ContentPlot[] }
  | { type: 'section'; value: ContentSection }
  | { type: 'static'; inputs: ContentStatic }
  | { type: 'table'; value: ContentTable }
  | { type: 'warning'; value: ContentWarning };

export interface ContentAnchor {
  inputs: {
    id?: string;
  }
}

export interface ContentImage {
  inputs: {
    id: string;
    mediacloudSmall: string;
    mediacloudNormal: string;
    mediacloudLarge: string;
    format: string;
    key: string;
    title: string;
    inTable?: boolean;
    legend?: [Children, Children][];
  }
}

export interface ContentInvoiceTable {
  data?: Children;
}

export interface ContentLaborpos {
  timeUnits: string;
  titleHtml: string;
  withHtml?: string;
  withoutHtml?: string;
}

export interface ContentList {
  inputs: {
    items: Children;
  }
}

export interface ContentLink {
  inputs: {
    html: string;
    id?: string;
    children?: Children;
  }
}

export interface ContentMixed {
  inputs: {
    children: Children;
  }
}

export interface ContentNested {
  inputs: {
    type: NestedType;
    documentType?: DocumentType;
    scenarioId?: string;
    aposNumber?: string;
    hkapId?: string;
    activity?: string;
    title?: string;
  }
}

export enum NestedType {
  Dok = "Dok",
}

export interface ContentParagraph {
  inputs: {
    children: Children;
  }
}

export interface ContentPlot {
  inputs: PlotItem[];
}

export interface PlotItem {
  position?: string;
  id?: string;
  children: Children;
}

export interface ContentSection {
  inputs: {
    children?: Children;
    titleUiText?: TitleUiText;
    title?: string;
  }
}

export enum TitleUiText {
  LblModelYear = "LblModelYear",
}

export interface ContentStatic {
  inputs: {
    html?: string;
    para?: boolean;
    translateIds?: TranslateIds;
    id?: string;
  };
}

export type TranslateIds = TranslateId[];

export enum TranslateId {
  LblCf = "LblCf",
  LblFes5 = "LblFes5",
  LblSa4 = "LblSa4",
  LblFrom = "LblFrom",
  LblMediaLongPm = "LblMediaLongPm",
  LblMediaLongWest = "LblMediaLongWest",
  LblMediaLongOwm = "LblMediaLongOwm",
  LblTo = "LblTo",
  LblAposNumber = "LblAposNumber",
  LblDescription = "LblDescription",
  LblInumber = "LblInumber",
  LblTechnicalValuesOverviewTableBaseValueCol = "LblTechnicalValuesOverviewTableBaseValueCol",
  LblTechnicalValuesOverviewTableDescriptionCol = "LblTechnicalValuesOverviewTableDescriptionCol",
  LblTechnicalValuesOverviewTableKindCol = "LblTechnicalValuesOverviewTableKindCol",
  LblTechnicalValuesOverviewTableToleranceCol = "LblTechnicalValuesOverviewTableToleranceCol",
  LblTechnicalValuesOverviewTableUsageLocationCol = "LblTechnicalValuesOverviewTableUsageLocationCol",
}

export interface ContentTable {
  inputs: {
    id?: string;
    usageLocation?: string;
    description?: string;
    kind?: string;
    baseValue?: string;
    tolerance1?: string[];
    tolerance2?: string[];
    pgwide?: boolean;
    data?: Children;
    header?: Children;
    title?: string;
    images?: Children;
  }
}

export interface ContentWarning {
  inputs: {
    category: string;
    consequences?: Children;
    source?: string;
    actions?: Children;
    children?: Children;
  }
}

export interface Laborpo {
  label: string;
  aposNumber: string;
  timeUnits: string;
}

export enum FileFormat {
  PDF = "pdf",
  XML = "xml",
}

export enum LanguageCode {
  EN_US = "en_US",
}

export interface MediacloudImageIds {
  mediacloudSmall: Mediacloud;
  mediacloudNormal: Mediacloud;
}

export type Mediacloud = string | string[];

export interface Part {
  partNumber: string;
  label: string;
  extendedLabel: string;
  quantity: string;
}

export interface QualityLineSegment {
  locationText: string;
  locationType: string;
  symptomText: string;
  symptomType: string;
}

export enum SourceSystem {
  EMPTY = "",
  REDSYS = "Redsys",
}

export interface Toc {
  label: string;
  anchor: string;
}

export interface Tool {
  id: string;
  label: string;
  kind: string;
  toolNumber: string;
  toolDisplayNumber: string;
}

export interface Techvalue {
  id?: string;
  usageLocation?: string;
  description?: string;
  kind?: string;
  baseValue?: string;
  tolerance1?: string[];
  tolerance2?: string[];
  images?: Children;
  html?: string;
}

export class WorkshopLiterature {
  fileFormat: FileFormat;
  languageCode: LanguageCode;
  variantId: number;
  hkapId: number;
  targetHkapId?: string;
  version?: number;
  latestVersion?: any;
  versionSourceSystem?: string;
  sourceSystem?: SourceSystem;
  kdnr: string;
  tiNumber?: string;
  publicationDate: string;
  modificationDate: number;
  title: string;
  fileName: string;
  documentType: DocumentType;
  content?: Children;
  toc?: Toc[];
  techvalues?: Techvalue[];
  mediacloudImageIds?: MediacloudImageIds;
  tools?: Tool[];
  mediaCloudFileId?: string;
  issueDate?: string;
  vehicleComponentWithDocumentIndex?: string;
  links?: any[];
  qualityLineSegment?: QualityLineSegment;
  parts?: Part[];
  laborpos?: Laborpo[];

  constructor(config: IWorkshopLiterature) {
    this.fileFormat = config.fileFormat;
    this.languageCode = config.languageCode;
    this.variantId = config.variantId;
    this.hkapId = config.hkapId;
    this.targetHkapId = config.targetHkapId;
    this.version = config.version;
    this.latestVersion = config.latestVersion;
    this.versionSourceSystem = config.versionSourceSystem;
    this.sourceSystem = config.sourceSystem;
    this.kdnr = config.kdnr;
    this.tiNumber = config.tiNumber;
    this.publicationDate = config.publicationDate;
    this.modificationDate = config.modificationDate;
    this.title = config.title;
    this.fileName = config.fileName;
    this.documentType = config.documentType;
    this.content = config.content;
    this.toc = config.toc;
    this.techvalues = config.techvalues;
    this.mediacloudImageIds = config.mediacloudImageIds;
    this.tools = config.tools;
    this.mediaCloudFileId = config.mediaCloudFileId;
    this.issueDate = config.issueDate;
    this.vehicleComponentWithDocumentIndex = config.vehicleComponentWithDocumentIndex;
    this.links = config.links;
    this.qualityLineSegment = config.qualityLineSegment;
    this.parts = config.parts;
    this.laborpos = config.laborpos;
  }
}

export enum DocumentType {
  // Mostly came from scraping API
  // Part-id specific media
  Mr = "Mr", // Manufacturer's Certificates - AKA MC
  Sit = "Sit", // Service Information Technik
  Sy = "Sy", // Symptom-based workshop manual
  Teq = "Teq", // Tequipment
  Ti = "Ti", // Technical Information
  Rm = "Rm", // Workshop Manual - AKA WM

  // Non part-id specific media
  Cam = "Cam", // Campaigns
  Diag = "Diag", // Diagnostic information
  Etm = "Etm", // Emission test
  Owm = "Owm", // Owner's Manuals
  Pm = "Pm", // Paint Manual
  Sf = "Sf", // Standard forms
  Tv = "Tv", // Technical values
  West = "West", // Workshop equipment

  // Country specific media
  Td = "Td", // PCNA Training Documentation

  // ??
  Faq = "Faq",
  Gfs = "Gfs",
  TsiCsdCn = "TsiCsdCn",
  TsiCsdUs = "TsiCsdUs",

  // these came from workshop lit
  Empty = "", // Representing empty value
  Hbw = "Hbw",
  Rl = "Rl",
  Tiaktion = "Tiaktion",
  Tiequip = "Tiequip",
  Ubb = "Ubb",
}

export interface IToolData {
  title: string;
  tool_number_pag: string;
  tool_number_vw?: string;
  tool_type: string;
  dealer_classification: string;
  description: string;
  hook_code?: string;
  model_series?: string;
  order_type?: string;
  tool_order_number?: string;
  state: string;
  distributors?: IToolDistributor[];
  referencing_documents?: IReferencingDocument[];
}

export class ToolData {
  title: string;
  tool_number_pag: string;
  tool_number_vw?: string;
  tool_type: string;
  dealer_classification: string;
  description: string;
  hook_code?: string;
  model_series?: string;
  order_type?: string;
  tool_order_number?: string;
  state: string;
  distributors?: ToolDistributor[];
  referencing_documents?: ReferencingDocument[];

  constructor(config: IToolData) {
    this.title = config.title;
    this.tool_number_pag = config.tool_number_pag;
    this.tool_number_vw = config.tool_number_vw;
    this.tool_type = config.tool_type;
    this.dealer_classification = config.dealer_classification;
    this.description = config.description;
    this.hook_code = config.hook_code;
    this.model_series = config.model_series;
    this.order_type = config.order_type;
    this.tool_order_number = config.tool_order_number;
    this.state = config.state;
    this.distributors = config.distributors;
    this.referencing_documents = config.referencing_documents;
  }
}

export interface IToolDistributor {
  id?: number;
  tool_id: number;
  part_number: string;
  distributor_code: string;
  name: string;
  city?: string;
  zip?: string;
  street?: string;
  phone?: string;
  fax?: string;
  email?: string;
  web?: string;
  creation_date?: string;
  modification_date?: string;
  key: string;
}

export class ToolDistributor {
  id?: number;
  tool_id: number;
  part_number: string;
  distributor_code: string;
  name: string;
  city?: string;
  zip?: string;
  street?: string;
  phone?: string;
  fax?: string;
  email?: string;
  web?: string;
  creation_date?: string;
  modification_date?: string;
  key: string;

  constructor(config: IToolDistributor) {
    this.id = config.id;
    this.tool_id = config.tool_id;
    this.part_number = config.part_number;
    this.distributor_code = config.distributor_code;
    this.name = config.name;
    this.city = config.city;
    this.zip = config.zip;
    this.street = config.street;
    this.phone = config.phone;
    this.fax = config.fax;
    this.email = config.email;
    this.web = config.web;
    this.creation_date = config.creation_date;
    this.modification_date = config.modification_date;
    this.key = config.key;
  }
}

export interface IReferencingDocument {
  hkap_id: string;
  variant_id: string;
  language_code: string;
  version: number;
  vehicle_component: string;
  title: string;
  document_type: string;
  publication_date: string;
  file_format: string;
  vehicle_component_with_document_index: string;
  new: boolean;
  bookmarked: boolean;
}

export class ReferencingDocument {
  hkap_id: string;
  variant_id: string;
  language_code: string;
  version: number;
  vehicle_component: string;
  title: string;
  document_type: string;
  publication_date: string;
  file_format: string;
  vehicle_component_with_document_index: string;
  new: boolean;
  bookmarked: boolean;

  constructor(config: IReferencingDocument) {
    this.hkap_id = config.hkap_id;
    this.variant_id = config.variant_id;
    this.language_code = config.language_code;
    this.version = config.version;
    this.vehicle_component = config.vehicle_component;
    this.title = config.title;
    this.document_type = config.document_type;
    this.publication_date = config.publication_date;
    this.file_format = config.file_format;
    this.vehicle_component_with_document_index = config.vehicle_component_with_document_index;
    this.new = config.new;
    this.bookmarked = config.bookmarked
  }
}