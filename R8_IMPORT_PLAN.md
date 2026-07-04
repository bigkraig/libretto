# Audi R8 (2016-2020) Workshop Manual — Import Plan

Source: `Workshop Manual AUDI R8 2016-2020/` — 22 PDFs, ~190 MB.

## Approach

Each PDF carries a clean PDF bookmark tree. The top level is the standard
VW/Audi **repair group** (`46 – Brakes - mechanism`); the level below it is the
**section** (`Front brakes`, `Rear brakes`). We split every PDF into
section-level documents and rebuild the tree in Libretto so each section becomes its
own searchable, viewable document — exactly like the existing Porsche/Ferrari
data.

## Vehicle key

One vehicle row: **year 2018, model `R8`**, name "R8 2016-2020".

## Tree shape (5 levels)

```
R8 (2018)                         root, location 000
└── Category                      8 buckets grouping the 22 manuals
    └── Manual                    the source PDF
        └── Repair group          e.g. "46 – Brakes - mechanism"
            └── Section           = one document  (PDF page-range slice)
```

## Two PDF structure types

- **Type A (19 PDFs)** — proper repair-group bookmarks. Split at the **section**
  level (level 1). → 318 documents.
- **Type B (3 PDFs)** — `Maintenance`, `Maintenance2`, `General Body Repairs`.
  Their bookmarks are checklist-style sub-items (e.g. "3.42 Horn: checking
  operation", many 0-1 pages). Splitting at that level would make ~300 junk
  docs, so we split at the **chapter** level (level 0). → 21 documents.

**Total: 339 documents across 8 categories.**

## Category → manual map

| Category | Manuals |
|---|---|
| Engine | 10-cylinder engine, Technical data for engines, Fuel supply system |
| Transmission & Final Drive | 7-speed DCT 0BZ, Front final drive 0D4 |
| Running Gear, Axles & Steering | Running gear/axles/steering, Wheels and tyres, Wheel Tyre Guide |
| Brakes | Brake system, Braking-force test spec |
| Body | Body Repairs, General Body Repairs [B], General body repairs interior |
| Heating & Air Conditioning | Heating & A/C, A/C R134a, A/C R1234yf |
| Electrical & Communication | Electrical system, Electrical system general, Communication, Radio fitting |
| Maintenance | Maintenance [B], Maintenance2 [B] |

The full per-section tree with page ranges is in `r8_proposed_tree.txt`.

## How each document is built

- Slice the source PDF to the section's page range → a standalone PDF.
- Store the slice in `media_images` keyed by a deterministic UUID.
- Create a `documents` row (`file_format=pdf`, title = section name,
  `document_type=MR`, `vehicle_component` = repair-group code).
- Create the tree nodes (category / manual / repair group) and link the doc.
- Run PDF text extraction (existing `extract-pdf-text`) so the section body is
  full-text searchable, including the squished-word fix.

## Open decisions (see review questions)

1. Keep the **Manual** level, or merge sections directly under the repair group
   within a category? (Merging is flatter but collides on repeated groups like
   "00 – Technical data" / "44 – Wheels" that appear in several manuals.)
2. Confirm Type-B (Maintenance / General Body Repairs) → chapter-level docs.
3. Keep the boilerplate "00 – Technical data" groups (identification / safety /
   repair instructions), or skip them?
