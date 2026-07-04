'use client';
import LibraryBooksIcon from '@mui/icons-material/LibraryBooks';
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined';
import SearchIcon from '@mui/icons-material/Search';
import ClearIcon from '@mui/icons-material/Clear';
import Tooltip from '@mui/material/Tooltip';
import React, {useEffect, useRef, useState} from "react";
import clsx from "clsx";
import {InsertDriveFile, PictureAsPdf} from "@mui/icons-material";
import {IDocument, ListDocuments, ListVehicleDocuments, SearchDocumentsByVehicle, SearchDocumentsInSubtree} from "@/lib/api";
import {useSearchParams} from "next/navigation";

const kinds: { [id: string]: string } = {
  "RM": "Workshop Manual",
  "TI": "Technical Information",
  "SIT": "Service Information Technik",
  "MC": "Manufacturer's Certificates",
  "TEQ": "Tequipment",
  "SY": "Symptom-based Workshop Manual",
}

// Brass chip for every document type — uniform (so RM doesn't read heavier than
// the rest) but keeps the brass character.
const BADGE_CLASS = "bg-brass-wash text-brass-dim";

function TypeBadge({type}: { type: string }) {
  const label = (type || "—").toUpperCase();
  const badge = (
    <span className={clsx(
      "inline-flex items-center justify-center min-w-[2.6rem] px-1.5 py-1 rounded-sm",
      "font-mono text-[11px] leading-none tracking-wide",
      BADGE_CLASS,
    )}>{label}</span>
  );
  return kinds[label]
    ? <Tooltip title={kinds[label]} placement="bottom-end">{badge}</Tooltip>
    : badge;
}

function InfoState({title, hint}: { title: string, hint?: string }) {
  return (
    <div className={clsx("flex flex-col items-center justify-center gap-1.5 px-6 py-12 text-center")}>
      <InfoOutlinedIcon className={clsx("text-line")} style={{fontSize: 34}}/>
      <div className={clsx("text-sm text-ink")}>{title}</div>
      {hint && <div className={clsx("text-xs text-muted max-w-xs")}>{hint}</div>}
    </div>
  )
}

const PAGE = 80;

function DocumentsList({documents}: { documents: IDocument[] }) {
  const searchParams = useSearchParams()
  const vehicle = searchParams.get("vehicle");
  const year = searchParams.get("year") ? Number(searchParams.get("year")) : null;

  // Render progressively — the vehicle-level list can be hundreds/thousands of rows.
  const [visible, setVisible] = useState(PAGE);
  const sentinelRef = useRef<HTMLDivElement>(null);

  useEffect(() => { setVisible(PAGE) }, [documents]);

  useEffect(() => {
    const el = sentinelRef.current;
    if (!el) return;
    // The list grows with content and the page scrolls (the panel is not a bounded
    // scroller), so observe against the viewport.
    const io = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) setVisible(v => Math.min(v + PAGE, documents.length));
      },
      {rootMargin: "500px"},
    );
    io.observe(el);
    return () => io.disconnect();
  }, [documents.length, visible]);

  if (vehicle == null || year == null) {
    return <InfoState title="Select a vehicle" hint="Choose a vehicle from the list to browse its service literature."/>
  }
  if (documents.length == 0) {
    return <InfoState title="No documents" hint="This vehicle has no workshop literature yet."/>
  }

  return (
    <div className={clsx("divide-y divide-line")}>
      {
        documents.slice(0, visible).map((doc: IDocument) => {
          const href = `/documents/${year}/${vehicle}/${doc.hkap_id}`
          return (
            <a key={doc.id}
               href={href}
               className={clsx(
                 "group flex items-center gap-3 px-4 py-2.5",
                 "hover:bg-brass-wash focus:bg-brass-wash focus-visible:outline-none transition-colors",
               )}>
              <TypeBadge type={doc.document_type}/>
              {doc.file_format == "pdf"
                ? <PictureAsPdf fontSize="small" className={clsx("shrink-0 text-muted group-hover:text-brass-dim")}/>
                : <InsertDriveFile fontSize="small" className={clsx("shrink-0 text-muted")}/>}
              <span className={clsx("shrink-0 font-mono text-[13px] text-brass-dim")}>
                {doc.vehicle_component_with_document_index}
              </span>
              <span className={clsx("truncate text-[14px] text-ink")}>{doc.title}</span>
            </a>
          );
        })
      }
      {visible < documents.length && <div ref={sentinelRef} className={clsx("h-10")}/>}
    </div>
  )
}

export type Params = {
  location: number | null,
  vehicle: string | null,
  year: number | null,
}

export default function Index({location, vehicle, year}: Params) {
  let [documents, setDocuments] = useState<IDocument[]>([])
  let [query, setQuery] = useState<string>("")
  let [searchResults, setSearchResults] = useState<IDocument[] | null>(null)
  let [searching, setSearching] = useState<boolean>(false)
  const searchAbortRef = useRef<AbortController | null>(null)

  useEffect(() => {
    if (!vehicle || !year) {
      setDocuments([])
      return
    }

    const d = document.querySelector(`#${documentsDiv}`)
    if (d) {
      d.scrollTop = Number(localStorage.getItem(key) || "")
    }

    // A component (location) shows its subtree; no component shows the whole vehicle.
    const req = location ? ListDocuments(location) : ListVehicleDocuments(year, vehicle)
    req.then(docs => setDocuments(docs)).catch(() => setDocuments([]))
  }, [location, vehicle, year])

  // Reset search when navigating to a different location.
  useEffect(() => {
    setQuery("")
    setSearchResults(null)
  }, [location, vehicle, year])

  // Debounced search. Subtree-scoped when a location is selected; vehicle-scoped otherwise.
  useEffect(() => {
    const q = query.trim()
    if (!vehicle || !year || q.length === 0) {
      searchAbortRef.current?.abort()
      setSearchResults(null)
      setSearching(false)
      return
    }

    const handle = setTimeout(() => {
      searchAbortRef.current?.abort()
      const ctrl = new AbortController()
      searchAbortRef.current = ctrl
      setSearching(true)
      const req = location
        ? SearchDocumentsInSubtree(location, q, ctrl.signal)
        : SearchDocumentsByVehicle(year, vehicle, q, ctrl.signal)
      req
        .then(docs => {
          if (!ctrl.signal.aborted) {
            setSearchResults(docs)
            setSearching(false)
          }
        })
        .catch(err => {
          if (err?.name === 'AbortError') return
          setSearchResults([])
          setSearching(false)
        })
    }, 250)

    return () => clearTimeout(handle)
  }, [query, location, vehicle, year])

  const key = "documents_position"
  const documentsDiv = "documents"

  const saveToLocalStorage = function () {
    const d = document.querySelector(`#${documentsDiv}`)
    if (d) {
      localStorage.setItem(key, d.scrollTop.toLocaleString())
    }
  }

  const isSearching = query.trim().length > 0
  const displayed = isSearching ? (searchResults ?? []) : documents
  const canSearch = !!(vehicle && year)
  const searchPlaceholder = !canSearch
    ? "Select a vehicle to search"
    : location
      ? "Search this section (title & content)"
      : "Search all documents for this vehicle (title & content)"

  return (
    <div className={clsx("w-full h-full flex flex-col bg-white border border-line overflow-hidden")}>
      <div className={clsx("flex w-full flex-wrap items-center justify-between gap-x-4 gap-y-2 border-b border-line bg-white px-4 py-2.5")}>

        <div className={clsx("flex w-full items-center gap-2.5 md:w-auto")}>
          <LibraryBooksIcon fontSize="small" className={clsx("text-muted")}/>
          <span className={clsx("font-mono text-[12px] uppercase tracking-[0.14em] text-ink")}>Workshop literature</span>
          {isSearching && searching && (
            <span className={clsx("flex items-center gap-1.5 text-xs text-muted")}>
              <span className={clsx("size-1.5 rounded-full bg-brass animate-pulse")}/> Searching…
            </span>
          )}
          <span className={clsx("ml-auto md:ml-0 font-mono text-[11px] leading-none px-2 py-1 rounded-full bg-ink text-white tabular-nums")}>{displayed.length}</span>
        </div>

        <div className={clsx("w-full md:w-auto")}>
          <div className={clsx("flex w-full flex-row items-center gap-1.5 rounded-sm border border-line bg-paper px-2 md:w-auto",
                               "focus-within:border-brass focus-within:ring-1 focus-within:ring-brass",
                               "aria-disabled:opacity-60")}
               aria-disabled={!canSearch}>
            <SearchIcon fontSize="small" className={clsx("text-muted")}/>
            <input autoComplete="off" type="search"
                   className={clsx("w-full bg-transparent py-1.5 outline-none placeholder:text-muted disabled:text-muted",
                                   "text-base md:w-[26rem] md:text-sm")}
                   placeholder={searchPlaceholder}
                   maxLength={100}
                   disabled={!canSearch}
                   value={query}
                   onChange={(e) => setQuery(e.target.value)}/>
            {query.length > 0 && (
              <button type="button"
                      className={clsx("text-muted hover:text-ink")}
                      onClick={() => setQuery("")}
                      aria-label="Clear search">
                <ClearIcon fontSize="small"/>
              </button>
            )}
          </div>
        </div>
      </div>

      <div className={clsx("w-full h-full flex flex-col overflow-y-auto")} id={documentsDiv}
           onClick={saveToLocalStorage}>
        {isSearching && !searching && (searchResults?.length ?? 0) === 0 ? (
          <InfoState title={`No documents match “${query}”`}
                     hint={location ? "Try a different term, or clear the section to search the whole vehicle." : "Try a different term."}/>
        ) : (
          <DocumentsList documents={displayed}/>
        )}
      </div>
    </div>
  );
}