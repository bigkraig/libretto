CREATE TABLE IF NOT EXISTS tree_node_links (
    id INTEGER PRIMARY KEY NOT NULL,
    parent_node_id INTEGER NOT NULL,
    child_node_id INTEGER NOT NULL,
    FOREIGN KEY(parent_node_id) REFERENCES tree_nodes(node_id),
    FOREIGN KEY(child_node_id) REFERENCES tree_nodes(node_id),
    CONSTRAINT unq UNIQUE (parent_node_id, child_node_id)
);

CREATE TABLE IF NOT EXISTS tree_nodes (
    node_id INTEGER PRIMARY KEY NOT NULL,
    vehicle TEXT NOT NULL,
    year INTEGER NOT NULL,
    node_value TEXT NOT NULL,
    name TEXT,
    illustration_id INTEGER NOT NULL,
    location TEXT,
    filter_applies INTEGER,
    CONSTRAINT unq UNIQUE (vehicle, year, node_id)
);

CREATE INDEX IF NOT EXISTS tree_nodes_idx ON tree_nodes (vehicle, year, node_value);

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
    new INTEGER NOT NULL,
    bookmarked INTEGER NOT NULL,
    content BLOB NOT NULL,
    CONSTRAINT unq UNIQUE (hkap_id)
);

CREATE INDEX IF NOT EXISTS documents_idx ON documents (hkap_id);

CREATE TABLE IF NOT EXISTS document_links (
    id INTEGER PRIMARY KEY NOT NULL,
    node_id INTEGER NOT NULL,
    hkap_id TEXT NOT NULL,
    FOREIGN KEY(node_id) REFERENCES tree_nodes(node_id),
    FOREIGN KEY(hkap_id) REFERENCES documents(hkap_id),
    CONSTRAINT unq UNIQUE (node_id, hkap_id)
);

CREATE INDEX IF NOT EXISTS document_links_idx ON document_links (node_id, hkap_id);

CREATE TABLE IF NOT EXISTS illustrations (
    illustration_id INTEGER PRIMARY KEY NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS parts (
    id INTEGER PRIMARY KEY NOT NULL,
    vehicle TEXT NOT NULL,
    year INTEGER NOT NULL,
    vehicle_component TEXT NOT NULL,
    part_id TEXT NOT NULL,
    paw_relevant INTEGER NOT NULL,
    text TEXT,
    CONSTRAINT unq UNIQUE (vehicle, year, vehicle_component, part_id)
);

CREATE INDEX IF NOT EXISTS parts_idx ON parts (vehicle, year, vehicle_component, part_id);

CREATE TABLE IF NOT EXISTS vehicles (
    id INTEGER PRIMARY KEY NOT NULL,
    year INTEGER NOT NULL,
    vehicle TEXT NOT NULL,
    name TEXT NOT NULL,
    image BLOB NOT NULL,
    CONSTRAINT unq UNIQUE (vehicle, year)
);

CREATE TABLE IF NOT EXISTS workshop_images (
    id INTEGER NOT NULL,
    size TEXT NOT NULL,
    content BLOB NOT NULL,
    CONSTRAINT unq UNIQUE (id, size),
    PRIMARY KEY (id, size)
);

CREATE TABLE IF NOT EXISTS media_images (
    id TEXT PRIMARY KEY NOT NULL,
    content BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_images (
    id TEXT PRIMARY KEY NOT NULL,
    content BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS translations (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tools (
    id INTEGER PRIMARY KEY NOT NULL,
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
    CONSTRAINT unq UNIQUE (tool_number_pag)
);

CREATE INDEX IF NOT EXISTS tools_idx ON tools (tool_number_pag);

CREATE TABLE IF NOT EXISTS tool_distributors (
    id INTEGER PRIMARY KEY NOT NULL,
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
    CONSTRAINT unq UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS tool_distributors_links (
    id INTEGER PRIMARY KEY NOT NULL,
    tool_id INTEGER NOT NULL,
    tool_distributor_id INTEGER NOT NULL,
    CONSTRAINT unq UNIQUE (tool_id, tool_distributor_id),
    FOREIGN KEY(tool_id) REFERENCES tools(id),
    FOREIGN KEY(tool_distributor_id) REFERENCES tool_distributors(id)
);

CREATE TABLE IF NOT EXISTS referencing_tool_documents (
    id INTEGER PRIMARY KEY NOT NULL,
    tool_id INTEGER NOT NULL,
    hkap_id TEXT NOT NULL,
    FOREIGN KEY(tool_id) REFERENCES tools(id),
    FOREIGN KEY(hkap_id) REFERENCES documents(hkap_id),
    CONSTRAINT unq UNIQUE (tool_id, hkap_id)
);

CREATE TABLE IF NOT EXISTS document_text (
    hkap_id TEXT PRIMARY KEY NOT NULL,
    text TEXT NOT NULL,
    text_squished TEXT NOT NULL DEFAULT '',
    FOREIGN KEY(hkap_id) REFERENCES documents(hkap_id)
);
