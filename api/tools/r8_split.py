#!/usr/bin/env python3
"""
Split the Audi R8 (2016-2020) workshop manual PDFs into section-level documents
and rebuild cross-reference links to point at the right Libretto document.

Tree (VW/Audi taxonomy):
    main group (0-9)
      repair group  "46 - Brakes - mechanism"   (tens digit of the RG number)
        [manual]                                 (only when the RG spans >1 manual)
          section  = document  (PDF page-range slice)

Repair group 00 ("Technical data") is generic front-matter, so it follows the
subject of its source manual (Brake system's 00 -> group 4, Engine's 00 -> 1).
Every other repair group maps by its own tens digit (69 -> 6, 94 -> 9).

Type-B manuals (no RG numbers): Maintenance -> group 0, General body repairs ->
group 5, each as a named node with its chapters as documents.

Output:
    <out>/<readable tree>/...pdf     one PDF per document
    <out>/manifest.json              full tree + document metadata (IDs owned here)

The Rust load_audi subcommand consumes the manifest + slices and writes the DB.

Links (all intra-PDF GoTo to named destinations):
    intra-section -> internal GoTo remapped to the page inside the slice
    cross-section -> URI  {ROOT}/documents/{year}/{model}/{hkap}#page=N
"""
import hashlib
import json
import os
import re
import sys
import uuid

from pypdf import PdfReader, PdfWriter
from pypdf.generic import (
    ArrayObject, DictionaryObject, NameObject, NumberObject, TextStringObject,
)

ROOT = "https://libretto.bigkraig.com"
YEAR = 2018
MODEL = "R8"
VEHICLE_NAME = "R8 2016-2020"

# Porsche/VAG main-group names
MAIN_NAMES = {
    0: "Entire vehicle – General",
    1: "Engine",
    2: "Fuel supply",
    3: "Transmission",
    4: "Chassis",
    5: "Body",
    6: "Interior equipment & occupant protection",
    7: "Trim, seats & insulation",
    8: "Air conditioning",
    9: "Electrical system",
}

# (filename, display label) - processing order; numbering comes from the taxonomy
ORDER = [
    ("Maintenance.pdf", "Maintenance"),
    ("Technical_data_for_engines.pdf", "Technical data for engines"),
    ("10-cylinder_direct_petrol_injection_engine_(5_2_ltr__4-valve).pdf", "10-cylinder engine (5.2 V10)"),
    ("Fuel_supply_system__petrol_engines.pdf", "Fuel supply system"),
    ("7-speed_dual_clutch_gearbox_0BZ.pdf", "7-speed dual clutch gearbox 0BZ"),
    ("Front_final_drive_0D4.pdf", "Front final drive 0D4"),
    ("Running_gear__axles__steering.pdf", "Running gear, axles & steering"),
    ("Wheels_and_tyres.pdf", "Wheels and tyres"),
    ("Wheel_Tyre_Guide.pdf", "Wheel & tyre guide"),
    ("Brake_system.pdf", "Brake system"),
    ("Specifications_for_testing_the_braking_force_(according_to_German_legislation).pdf", "Braking-force test spec"),
    ("Body_Repairs.pdf", "Body repairs"),
    ("Body_Repairs__General_Body_Repairs.pdf", "General body repairs"),
    ("General_body_repairs__interior.pdf", "Body repairs - interior"),
    ("Heating__air_conditioning.pdf", "Heating & air conditioning"),
    ("Air_conditioner_with_refrigerant_R134a.pdf", "A/C - refrigerant R134a"),
    ("Air_conditioners_with_refrigerant_R1234yf_-_General_information.pdf", "A/C - refrigerant R1234yf"),
    ("Electrical_system.pdf", "Electrical system"),
    ("Electrical_system__General_information.pdf", "Electrical system - general"),
    ("Communication.pdf", "Communication"),
    ("Fitting_instructions__radio_communication_systems.pdf", "Radio comm. fitting"),
]

# manuals with no repair-group numbers -> placed by subject
TYPE_B_MAIN = {
    "Maintenance.pdf": 0,
    "Body_Repairs__General_Body_Repairs.pdf": 5,
}
# manuals that are 00-only (or otherwise need an explicit subject for their 00)
SUBJECT_OVERRIDE = {
    "Technical_data_for_engines.pdf": 1,
    "Specifications_for_testing_the_braking_force_(according_to_German_legislation).pdf": 4,
}

RG_RE = re.compile(r"^\s*(\d+)\s*[–-]\s*(.*)$")          # "46 – Brakes - mechanism"
NUM_PREFIX_RE = re.compile(r"^\d+(\.\d+)*\s+")           # "3.42 Horn..." / "1 Front brakes"


def node_id_for(location: str) -> int:
    h = hashlib.sha1(f"audi-{MODEL}-{YEAR}-{location}".encode()).digest()
    return int.from_bytes(h[:4], "big") & 0x7FFFFFFF


def media_uuid_for(hkap_id: str) -> str:
    return str(uuid.uuid5(uuid.NAMESPACE_DNS, f"audi-{MODEL}-{YEAR}-{hkap_id}"))


def clean_title(t: str) -> str:
    return NUM_PREFIX_RE.sub("", t).strip()


def sanitize(name: str) -> str:
    name = name.replace("/", "-").replace(":", " -").replace("\\", "-")
    name = re.sub(r'[<>:"|?*\x00-\x1f]', "", name)
    name = re.sub(r"\s+", " ", name).strip(" .")
    return name[:120]


def read_outline(reader):
    out = []

    def walk(items, depth=0):
        for it in items:
            if isinstance(it, list):
                walk(it, depth + 1)
            else:
                try:
                    page = reader.get_destination_page_number(it)
                except Exception:
                    page = None
                out.append((depth, (it.title or "").strip(), page))
    try:
        walk(reader.outline)
    except Exception:
        pass
    return out


def _dest_top(d):
    """Extract the /XYZ top Y-coordinate from a destination array, or None."""
    try:
        if isinstance(d, list) and len(d) >= 4 and str(d[1]) == "/XYZ":
            t = d[3]
            return float(t)
    except (TypeError, ValueError):
        pass
    return None


def named_dests(reader):
    """name -> (abs_page_index, top_y_or_None)."""
    idx = {p.indirect_reference: i for i, p in enumerate(reader.pages)}
    out = {}
    names = reader.trailer["/Root"].get("/Names")
    if not names or not names.get("/Dests"):
        return out

    def walk(node):
        node = node.get_object()
        if "/Names" in node:
            arr = node["/Names"]
            for k in range(0, len(arr), 2):
                dd = arr[k + 1].get_object()
                d = dd.get("/D") if isinstance(dd, dict) else dd
                page = idx.get(d[0]) if isinstance(d, list) and d else None
                out[str(arr[k])] = (page, _dest_top(d))
        for kid in node.get("/Kids", []):
            walk(kid)

    walk(names["/Dests"])
    return out


def extract_groups(reader):
    """Return (is_A, groups). Each group: {num, name, sections:[{title,start,end}]}.
    Type-B manuals get one synthetic group (num=None) whose 'sections' are chapters."""
    npages = len(reader.pages)
    flat = read_outline(reader)
    groups, cur = [], None
    for depth, title, page in flat:
        if depth == 0:
            cur = {"title": title, "page": page, "sections": []}
            groups.append(cur)
        elif depth == 1 and cur is not None:
            cur["sections"].append({"title": title, "page": page})
    for gi, g in enumerate(groups):
        g["end"] = (groups[gi + 1]["page"] - 1) if gi + 1 < len(groups) and groups[gi + 1]["page"] is not None else npages - 1
        for si, s in enumerate(g["sections"]):
            nxt = g["sections"][si + 1]["page"] if si + 1 < len(g["sections"]) else None
            s["end"] = (nxt - 1) if nxt is not None else g["end"]

    is_A = len(groups) > 0 and all(RG_RE.match(g["title"]) for g in groups)
    norm = []
    if is_A:
        # coalesce repeated repair-group numbers within the manual (e.g. 39 twice)
        by_num, order = {}, []
        for g in groups:
            m = RG_RE.match(g["title"])
            num, name = int(m.group(1)), m.group(2).strip()
            secs = [{"title": clean_title(s["title"]), "start": s["page"], "end": s["end"]}
                    for s in g["sections"] if s["page"] is not None]
            if num in by_num:
                by_num[num]["sections"] += secs
                by_num[num]["_names"].append(name)
            else:
                by_num[num] = {"num": num, "name": name, "sections": secs, "_names": [name]}
                order.append(num)
        for num in order:
            e = by_num[num]
            if len(e["_names"]) > 1:
                e["name"] = common_name(e["_names"])
            norm.append({"num": e["num"], "name": e["name"], "sections": e["sections"]})
    else:
        # type B: each level-0 chapter is a document
        norm.append({
            "num": None, "name": None,
            "sections": [{"title": clean_title(g["title"]), "start": g["page"], "end": g["end"]}
                         for g in groups if g["page"] is not None],
        })
    return is_A, norm


def dominant_main(groups):
    nz = [g["num"] // 10 for g in groups if g["num"] not in (None, 0)]
    if not nz:
        return None
    return max(set(nz), key=nz.count)


def common_name(names):
    names = sorted(set(names))
    if len(names) == 1:
        return names[0]
    pref = os.path.commonprefix(names).rstrip(" -")
    return pref or names[0]


def page_to_doc(docs, abs_page):
    best = None
    for d in docs:
        if d["start"] <= abs_page <= d["end"]:
            return d
        if d["start"] <= abs_page and (best is None or d["start"] > best["start"]):
            best = d
    return best or (docs[0] if docs else None)


def _resolve_target(o, idx, dests):
    """Return (abs_page_index, top_y_or_None) for a link annotation, or None."""
    A = o.get("/A")
    d = A.get("/D") if A and A.get("/S") == "/GoTo" else o.get("/Dest")
    if d is None:
        return None
    if isinstance(d, list) and d:
        return (idx.get(d[0]), _dest_top(d))
    return dests.get(str(d))


def slice_and_relink(src_path, doc, manual_docs, dests, out_dir):
    """Slice one document's pages and rebuild its links. Returns (media_id, npages).

    pypdf.append() drops GoTo links whose target is outside the copied range, so we
    capture links from the source first, then rebuild them on the slice."""
    reader = PdfReader(src_path)
    idx = {p.indirect_reference: i for i, p in enumerate(reader.pages)}
    start, end = doc["start"], doc["end"]

    # page-height lookup (for converting a dest's PDF-space Y into a top fraction)
    def top_fraction(abs_page, top):
        if top is None:
            return None
        box = reader.pages[abs_page].mediabox
        height = float(box.height)
        if height <= 0:
            return None
        frac = (float(box.top) - top) / height
        return max(0.0, min(1.0, frac))

    captured = []
    for src_i in range(start, end + 1):
        links = []
        for a in (reader.pages[src_i].get("/Annots") or []):
            o = a.get_object()
            if o.get("/Subtype") != "/Link":
                continue
            rect, res = o.get("/Rect"), _resolve_target(o, idx, dests)
            if rect is None or res is None or res[0] is None:
                continue
            tgt, top = res
            links.append(([float(x) for x in rect], tgt, top))
        captured.append(links)

    writer = PdfWriter()
    writer.append(reader, pages=(start, end + 1))

    for local_i, page in enumerate(writer.pages):
        kept = [a for a in (page.get("/Annots") or [])
                if a.get_object().get("/Subtype") != "/Link"]
        annots = ArrayObject(kept)
        for rect_vals, tgt, top in captured[local_i]:
            target = page_to_doc(manual_docs, tgt)
            if target is None:
                continue
            page_off = tgt - target["start"] + 1
            frac = top_fraction(tgt, top)
            frag = f"#page={page_off}" + (f"&y={frac:.4f}" if frac is not None else "")
            # Every link (intra- and cross-document) is a full, shareable URL.
            # An intra-doc link differs from the current URL only by fragment, so
            # the browser scrolls without reloading.
            url = f"{ROOT}/documents/{YEAR}/{MODEL}/{target['hkap_id']}{frag}"
            action = DictionaryObject()
            action[NameObject("/S")] = NameObject("/URI")
            action[NameObject("/URI")] = TextStringObject(url)
            annot = DictionaryObject()
            annot[NameObject("/Type")] = NameObject("/Annot")
            annot[NameObject("/Subtype")] = NameObject("/Link")
            annot[NameObject("/Rect")] = ArrayObject([NumberObject(v) for v in rect_vals])
            annot[NameObject("/Border")] = ArrayObject([NumberObject(0)] * 3)
            annot[NameObject("/A")] = action
            annots.append(writer._add_object(annot))
        if len(annots):
            page[NameObject("/Annots")] = annots
        elif "/Annots" in page:
            del page[NameObject("/Annots")]

    media_id = media_uuid_for(doc["hkap_id"])
    out_path = os.path.join(out_dir, doc["rel_path"])
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "wb") as f:
        writer.write(f)
    return media_id, len(writer.pages)


def main():
    src_dir = sys.argv[1] if len(sys.argv) > 1 else "source_files/audi/2018-R8"
    out_dir = sys.argv[2] if len(sys.argv) > 2 else "source_files/audi/2018-R8-split"
    os.makedirs(out_dir, exist_ok=True)

    # ---- Pass A: scan manuals, decide main group per group, tally RG -> manuals
    scanned = {}      # file -> {"label","is_A","placed":[(mg, key, name, group)]}
    rg_manuals = {}   # key -> [files...]  (key = (mg,num) for A, (mg,"B:"+file) for B)
    rg_names = {}     # key -> [names]
    for fname, label in ORDER:
        reader = PdfReader(os.path.join(src_dir, fname))
        is_A, groups = extract_groups(reader)
        subject = SUBJECT_OVERRIDE.get(fname, dominant_main(groups)) if is_A else TYPE_B_MAIN[fname]
        placed = []
        for g in groups:
            if is_A:
                mg = subject if g["num"] == 0 else g["num"] // 10
                key = (mg, g["num"])
                rg_names.setdefault(key, []).append(g["name"])
            else:
                mg = TYPE_B_MAIN[fname]
                key = (mg, "B:" + fname)
                rg_names.setdefault(key, []).append(label)
            placed.append((mg, key, g))
            rg_manuals.setdefault(key, [])
            if fname not in rg_manuals[key]:
                rg_manuals[key].append(fname)
        scanned[fname] = {"label": label, "is_A": is_A, "placed": placed}

    # ---- Pass B: build tree
    root_loc = "000"
    manifest = {
        "vehicle": {"year": YEAR, "model": MODEL, "name": VEHICLE_NAME},
        "root": {"node_id": node_id_for(root_loc), "location": root_loc,
                 "node_value": "000", "name": MODEL, "parent_node_id": None},
        "nodes": [], "documents": [],
    }
    docs_by_manual = {f: [] for f, _ in ORDER}

    def add_node(node_id, parent, location, node_value, name):
        manifest["nodes"].append({"node_id": node_id, "parent_node_id": parent,
                                  "location": location, "node_value": node_value, "name": name})

    mains = sorted({mg for key in rg_manuals for (mg, _) in [key]})
    for mg in mains:
        mg_loc = f"000{mg:03d}"
        mg_node = node_id_for(mg_loc)
        mg_dir = sanitize(f"{mg} {MAIN_NAMES[mg]}")
        # node_value carries the number; the UI renders "{node_value} {name}"
        add_node(mg_node, manifest["root"]["node_id"], mg_loc, str(mg), MAIN_NAMES[mg])

        # order repair groups within the main group: numbered first (by num), type-B last
        keys = [k for k in rg_manuals if k[0] == mg]
        keys.sort(key=lambda k: (isinstance(k[1], str), k[1] if isinstance(k[1], int) else 0))
        for ri, key in enumerate(keys, start=1):
            num = key[1]
            rg_loc = f"{mg_loc}{ri:03d}"
            rg_node = node_id_for(rg_loc)
            if isinstance(num, int):
                rname = common_name(rg_names[key])
                rg_value = f"{num:02d}"
                rg_label = sanitize(f"{num:02d} {rname}")
                leaf_prefix = f"{num:02d}"
                vehicle_component = f"{num:02d}"
            else:  # type B (no repair-group number)
                rname = rg_names[key][0]
                rg_value = ""
                rg_label = sanitize(rname)
                leaf_prefix = ""
                vehicle_component = sanitize(rname)
            add_node(rg_node, mg_node, rg_loc, rg_value, rname)

            files = rg_manuals[key]
            multi = len(files) > 1
            for mi, fname in enumerate(files, start=1):
                # this manual's group object for this key
                grp = next(g for (m, k, g) in scanned[fname]["placed"] if k == key)
                if multi:
                    sub_loc = f"{rg_loc}{mi:03d}"
                    sub_node = node_id_for(sub_loc)
                    # manual sub-node: no code, name only
                    add_node(sub_node, rg_node, sub_loc, "", scanned[fname]["label"])
                    sec_parent, sec_base_loc = sub_node, sub_loc
                    path_prefix = f"{mg_dir}/{rg_label}/{sanitize(scanned[fname]['label'])}"
                else:
                    sec_parent, sec_base_loc = rg_node, rg_loc
                    path_prefix = f"{mg_dir}/{rg_label}"

                for si, s in enumerate(grp["sections"], start=1):
                    sec_loc = f"{sec_base_loc}{si:03d}"
                    hkap = f"audi{MODEL.lower()}-{sec_loc}"
                    rel = f"{path_prefix}/{si:02d} {sanitize(s['title'])}.pdf"
                    leaf_value = f"{leaf_prefix}.{si}" if leaf_prefix else str(si)
                    # when a repair group merges several manuals, tag the title with
                    # its source so flattened listings stay distinguishable
                    title = s["title"]
                    if multi:
                        title = f"{title} ({scanned[fname]['label']})"
                    doc = {
                        "hkap_id": hkap, "node_id": node_id_for(sec_loc),
                        "parent_node_id": sec_parent, "location": sec_loc,
                        "node_value": leaf_value, "title": title,
                        "vehicle_component": vehicle_component,
                        "start": s["start"], "end": s["end"], "rel_path": rel,
                        "source": fname,
                    }
                    docs_by_manual[fname].append(doc)

    # ---- Pass C: slice + relink per manual
    total = 0
    for fname, label in ORDER:
        src = os.path.join(src_dir, fname)
        mdocs = docs_by_manual[fname]
        dests = named_dests(PdfReader(src))
        for doc in mdocs:
            media_id, npages = slice_and_relink(src, doc, mdocs, dests, out_dir)
            manifest["documents"].append({
                "hkap_id": doc["hkap_id"], "node_id": doc["node_id"],
                "parent_node_id": doc["parent_node_id"], "location": doc["location"],
                "node_value": doc["node_value"], "title": doc["title"],
                "vehicle_component": doc["vehicle_component"], "media_id": media_id,
                "pdf": doc["rel_path"], "pages": npages, "source": fname,
            })
            total += 1
        print(f"{label}: {len(mdocs)} docs")

    with open(os.path.join(out_dir, "manifest.json"), "w") as f:
        json.dump(manifest, f, indent=2)
    print(f"\nWrote {total} documents, {len(manifest['nodes'])} folder nodes")


if __name__ == "__main__":
    main()
