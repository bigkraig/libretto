'use client';
import LibraryBooksIcon from '@mui/icons-material/LibraryBooks';
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined';
import SearchIcon from '@mui/icons-material/Search';
import ClearIcon from '@mui/icons-material/Clear';
import Tooltip from '@mui/material/Tooltip';
import React, {ReactElement, useEffect, useRef, useState} from "react";
import clsx from "clsx";
import {InsertDriveFile, PictureAsPdf} from "@mui/icons-material";
import {IDocument, ListDocuments, SearchDocumentsByVehicle, SearchDocumentsInSubtree} from "@/lib/api";
import {useSearchParams} from "next/navigation";

function NoDocuments() {
  return (
    <div className={clsx("pl-2 w-full flex bg-zinc-100")}>
      <div className={clsx("bg-zinc-700 p-1 size-11 flex")}>
        <InfoOutlinedIcon className={clsx("m-auto text-white")}/>
      </div>
      <div className={clsx("pl-2 pr-6 my-auto")}> Please select a component.</div>
    </div>
  )
}

const kinds: { [id: string]: string } = {
  "RM": "Workshop Manual",
  "TI": "Technical Information",
  "SIT": "Service Information Technik",
  "MC": "Manufacturer's Certificates",
  "TEQ": "Tequipment",
  "SY": "Symptom-based Workshop Manual",
}

function KindToolTip(props: {
  children: ReactElement<any, any>
  kind: string
}): React.ReactNode {
  return <Tooltip title={kinds[props.kind]} placement="bottom-end">
    {props.children}
  </Tooltip>
}

function DocumentsList({documents}: { documents: IDocument[] }) {
  const searchParams = useSearchParams()
  const vehicle = searchParams.get("vehicle");
  const year = searchParams.get("year") ? Number(searchParams.get("year")) : null;

  if (documents.length == 0 || vehicle == null || year == null) {
    return <NoDocuments/>
  }

  return (
    <div className={clsx("divide-y")}>
      {
        documents.map((doc: IDocument) => {
          const href = `/documents/${year}/${vehicle}/${doc.hkap_id}`
          return (
            <a key={doc.id} className={clsx("flex")} href={href}>
              <div className={clsx("pl-2 pt-2")}></div>
              <div className={clsx("p-2 bg-zinc-500 text-white size-11 flex")}>
                <KindToolTip kind={doc.document_type}>
                  <div className={clsx("m-auto")}>{doc.document_type}</div>
                </KindToolTip>
              </div>
              {doc.file_format == "pdf" ? <PictureAsPdf className={clsx("ml-4 my-auto text-black")}/> :
                <InsertDriveFile className={clsx("ml-4 my-auto text-black")}/>}
              <div className={clsx("ml-4 my-auto text-left")}>{doc.vehicle_component_with_document_index} - {doc.title}</div>
            </a>
          );
        })
      }
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
    if (!vehicle || !year || !location) {
      setDocuments([])
      return
    }

    const d = document.querySelector(`#${documentsDiv}`)
    if (d) {
      d.scrollTop = Number(localStorage.getItem(key) || "")
    }

    ListDocuments(location).then(docs => setDocuments(docs));
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
    <div className={clsx("w-full h-full flex flex-col bg-zinc-100")}>
      <div className={clsx("bg-zinc-200 flex w-full justify-between")}>

        <div className={clsx("px-4 py-2 flex")}>
          <LibraryBooksIcon className={clsx("my-auto")}/>
          <div className={clsx("pl-2 pr-6 my-auto font-bold")}> Workshop literature</div>
          <span className={clsx("pl-2 pr-2 bg-zinc-700 rounded-full text-white my-auto")}>{displayed.length}</span>
          {isSearching && searching && (
            <span className={clsx("pl-3 my-auto text-zinc-500 text-sm")}>Searching…</span>
          )}
        </div>

        <div className={clsx("px-4 py-2 flex flex-row justify-end")}>
          <div className={clsx("outline outline-1 flex flex-row bg-zinc-100 items-center")}>
            <div className={clsx("my-auto pl-1 text-zinc-600")}><SearchIcon fontSize="small"/></div>
            <input autoComplete="off" type="search"
                   className={clsx("bg-zinc-100 px-2 py-1 outline-none w-[28rem] disabled:text-zinc-400")}
                   placeholder={searchPlaceholder}
                   maxLength={100}
                   disabled={!canSearch}
                   value={query}
                   onChange={(e) => setQuery(e.target.value)}/>
            {query.length > 0 && (
              <button type="button"
                      className={clsx("px-1 text-zinc-600 hover:text-black")}
                      onClick={() => setQuery("")}
                      aria-label="Clear search">
                <ClearIcon fontSize="small"/>
              </button>
            )}
          </div>
        </div>
      </div>

      <div className={clsx("p-1")}></div>
      <div className={clsx("w-full h-full flex flex-col divide-y overflow-scroll")} id={documentsDiv}
           onClick={saveToLocalStorage}>
        {isSearching && !searching && (searchResults?.length ?? 0) === 0 ? (
          <div className={clsx("pl-2 w-full flex bg-zinc-100")}>
            <div className={clsx("bg-zinc-700 p-1 size-11 flex")}>
              <InfoOutlinedIcon className={clsx("m-auto text-white")}/>
            </div>
            <div className={clsx("pl-2 pr-6 my-auto")}>No documents match &ldquo;{query}&rdquo; in this section.</div>
          </div>
        ) : (
          <DocumentsList documents={displayed}/>
        )}
      </div>
    </div>
  );
}