CREATE TABLE IF NOT EXISTS tree_nodes (
    node_id INTEGER PRIMARY KEY NOT NULL,
    vehicle TEXT NOT NULL,
    year INTEGER NOT NULL,
    node_value TEXT NOT NULL,
    name TEXT,
    illustration_id INTEGER NOT NULL,
    location TEXT,
    filter_applies BOOLEAN,
    CONSTRAINT tree_nodes_unq UNIQUE (vehicle, year, node_id)
);

CREATE INDEX IF NOT EXISTS tree_nodes_idx ON tree_nodes (vehicle, year, node_value);

CREATE TABLE IF NOT EXISTS tree_node_links (
    id SERIAL PRIMARY KEY NOT NULL,
    parent_node_id INTEGER NOT NULL REFERENCES tree_nodes(node_id),
    child_node_id INTEGER NOT NULL REFERENCES tree_nodes(node_id),
    CONSTRAINT tree_node_links_unq UNIQUE (parent_node_id, child_node_id)
);

CREATE TABLE IF NOT EXISTS documents (
    hkap_id TEXT PRIMARY KEY NOT NULL,
    variant_id TEXT NOT NULL,
    language_code TEXT NOT NULL,
    version INTEGER NOT NULL,
    vehicle_component TEXT NOT NULL,
    title TEXT NOT NULL,
    document_type TEXT NOT NULL,
    publication_date TEXT NOT NULL,
    file_format TEXT NOT NULL,
    vehicle_component_with_document_index TEXT NOT NULL,
    new BOOLEAN NOT NULL,
    bookmarked BOOLEAN NOT NULL,
    content BYTEA NOT NULL,
    CONSTRAINT documents_unq UNIQUE (hkap_id)
);

CREATE INDEX IF NOT EXISTS documents_idx ON documents (hkap_id);

CREATE TABLE IF NOT EXISTS document_links (
    id SERIAL PRIMARY KEY NOT NULL,
    node_id INTEGER NOT NULL REFERENCES tree_nodes(node_id),
    hkap_id TEXT NOT NULL REFERENCES documents(hkap_id),
    CONSTRAINT document_links_unq UNIQUE (node_id, hkap_id)
);

CREATE INDEX IF NOT EXISTS document_links_idx ON document_links (node_id, hkap_id);

CREATE TABLE IF NOT EXISTS illustrations (
    illustration_id INTEGER PRIMARY KEY NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS parts (
    id SERIAL PRIMARY KEY NOT NULL,
    vehicle TEXT NOT NULL,
    year INTEGER NOT NULL,
    vehicle_component TEXT NOT NULL,
    part_id TEXT NOT NULL,
    paw_relevant BOOLEAN NOT NULL,
    text TEXT,
    CONSTRAINT parts_unq UNIQUE (vehicle, year, vehicle_component, part_id)
);

CREATE INDEX IF NOT EXISTS parts_idx ON parts (vehicle, year, vehicle_component, part_id);

CREATE TABLE IF NOT EXISTS vehicles (
    id SERIAL PRIMARY KEY NOT NULL,
    year INTEGER NOT NULL,
    vehicle TEXT NOT NULL,
    name TEXT NOT NULL,
    image BYTEA NOT NULL,
    CONSTRAINT vehicles_unq UNIQUE (vehicle, year)
);

CREATE TABLE IF NOT EXISTS workshop_images (
    id INTEGER NOT NULL,
    size TEXT NOT NULL,
    content BYTEA NOT NULL,
    PRIMARY KEY (id, size)
);

CREATE TABLE IF NOT EXISTS media_images (
    id TEXT PRIMARY KEY NOT NULL,
    content BYTEA NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_images (
    id TEXT PRIMARY KEY NOT NULL,
    content BYTEA NOT NULL
);

CREATE TABLE IF NOT EXISTS translations (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tools (
    id SERIAL PRIMARY KEY NOT NULL,
    tool_number_pag TEXT NOT NULL,
    title TEXT NOT NULL,
    tool_number_vw TEXT,
    tool_type TEXT NOT NULL,
    dealer_classification TEXT NOT NULL,
    description TEXT NOT NULL,
    hook_code TEXT,
    model_series TEXT,
    order_type TEXT,
    tool_order_number TEXT,
    state TEXT NOT NULL,
    CONSTRAINT tools_unq UNIQUE (tool_number_pag)
);

CREATE INDEX IF NOT EXISTS tools_idx ON tools (tool_number_pag);

CREATE TABLE IF NOT EXISTS tool_distributors (
    id SERIAL PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    part_number TEXT NOT NULL,
    distributor_code TEXT NOT NULL,
    city TEXT,
    zip TEXT,
    street TEXT,
    phone TEXT,
    fax TEXT,
    email TEXT,
    web TEXT,
    creation_date TEXT,
    modification_date TEXT,
    key TEXT NOT NULL,
    CONSTRAINT tool_distributors_unq UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS tool_distributors_links (
    id SERIAL PRIMARY KEY NOT NULL,
    tool_id INTEGER NOT NULL REFERENCES tools(id),
    tool_distributor_id INTEGER NOT NULL REFERENCES tool_distributors(id),
    CONSTRAINT tool_distributors_links_unq UNIQUE (tool_id, tool_distributor_id)
);

CREATE TABLE IF NOT EXISTS referencing_tool_documents (
    id SERIAL PRIMARY KEY NOT NULL,
    tool_id INTEGER NOT NULL REFERENCES tools(id),
    hkap_id TEXT NOT NULL REFERENCES documents(hkap_id),
    CONSTRAINT referencing_tool_documents_unq UNIQUE (tool_id, hkap_id)
);

CREATE TABLE IF NOT EXISTS document_text (
    hkap_id TEXT PRIMARY KEY NOT NULL REFERENCES documents(hkap_id),
    text TEXT NOT NULL,
    text_squished TEXT NOT NULL DEFAULT ''
);
