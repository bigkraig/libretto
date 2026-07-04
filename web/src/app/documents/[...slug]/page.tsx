'use client';

import {usePathname} from "next/navigation";
import React, {useEffect, useState} from "react";
import {ArrowBack} from "@mui/icons-material";
import {SideBarIndex} from "@/components/SideBarIndex";
import clsx from "clsx";
import {GetTranslations, GetWorkshopLiterature, IWorkshopLiterature} from "@/lib/api";
import {WorkshopLiterature} from "@/components/WorkshopLiterature";
import {DocumentContext} from "@/lib";

function back() {
  window.history.go(-1);
  return false;
}

export default function Home() {
  let parts = usePathname().split("?")[0].split("/");
  let hkap_id = parts[4];

  let [workshopLiterature, setWorkshopLiterature] = useState<IWorkshopLiterature>()
  let [translations, setTranslations] = useState<Map<string, string>>()

  const [isToolsTableVisible, setToolsTableVisible] = useState(false);

  function toggleToolsTableVisible() {
    setToolsTableVisible(v => (!v));
  }

  const [isTechValuesTableVisible, setTechValuesTableVisible] = useState(false);

  function toggleTechValuesTableVisible() {
    setTechValuesTableVisible(v => (!v));
  }

  const [isPartsTableVisible, setPartsTableVisible] = useState(false);

  function togglePartsTableVisible() {
    setPartsTableVisible(v => (!v));
  }

  useEffect(() => {
    if (!hkap_id) {
      return
    }

    GetWorkshopLiterature(hkap_id)
      .then(content => setWorkshopLiterature(content))
      .catch(err => console.error('Failed to load workshop literature:', err));
    GetTranslations()
      .then(content => setTranslations(content))
      .catch(err => console.error('Failed to load translations:', err));
  }, [hkap_id])

  if (!workshopLiterature || !translations) {
    return (
      <div className={clsx("min-h-screen bg-paper")}>
        <div className={clsx("flex items-center gap-3 border-b border-line bg-white px-3 py-2.5")}>
          <div className={clsx("size-9 shrink-0 rounded-sm bg-ink/90")}/>
          <div className={clsx("h-4 w-56 max-w-[60vw] animate-pulse rounded bg-line")}/>
        </div>
        <div className={clsx("mx-auto max-w-3xl space-y-3 p-6")}>
          <div className={clsx("h-6 w-1/2 animate-pulse rounded bg-line")}/>
          <div className={clsx("h-4 w-full animate-pulse rounded bg-line")}/>
          <div className={clsx("h-4 w-11/12 animate-pulse rounded bg-line")}/>
          <div className={clsx("h-4 w-4/5 animate-pulse rounded bg-line")}/>
        </div>
      </div>
    )
  }

  const dtype = (workshopLiterature.documentType || "").toUpperCase();
  const badgeCls = "bg-brass-wash text-brass-dim";

  return (
    <DocumentContext.Provider
      value={{
        isToolsTableVisible,
        toggleToolsTableVisible,
        isTechValuesTableVisible,
        toggleTechValuesTableVisible,
        isPartsTableVisible,
        togglePartsTableVisible
      }}>
      <div className={clsx("min-h-screen bg-paper")}>
        <header className={clsx("sticky top-0 z-20 flex items-center gap-3 border-b border-line bg-white px-3 py-2.5 print:hidden")}>
          <button onClick={back} aria-label="Back"
                  className={clsx("inline-flex size-9 shrink-0 items-center justify-center rounded-sm bg-ink text-white",
                                  "transition-colors hover:bg-brass focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brass")}>
            <ArrowBack fontSize="small"/>
          </button>
          <span className={clsx("inline-flex items-center justify-center min-w-[2.6rem] px-1.5 py-1 rounded-sm font-mono text-[11px] leading-none tracking-wide", badgeCls)}>
            {dtype || "—"}
          </span>
          {workshopLiterature.kdnr && (
            <span className={clsx("shrink-0 font-mono text-[13px] text-brass-dim")}>{workshopLiterature.kdnr}</span>
          )}
          <span className={clsx("truncate text-[15px] font-medium text-ink")}>{workshopLiterature.title}</span>
        </header>

        <div className={clsx("grid grid-cols-6")}>
          <main className={clsx("col-span-6 lg:col-span-5 p-0 lg:p-2 print:overflow-visible")}>
            <WorkshopLiterature hkap_id={hkap_id} translations={translations}/>
          </main>
          <aside id="sideBar" className={clsx("hidden lg:block lg:col-span-1 border-l border-line bg-white print:hidden")}>
            <div className={clsx("sticky top-14")}>
              <SideBarIndex toc={workshopLiterature.toc}></SideBarIndex>
            </div>
          </aside>
        </div>
      </div>
    </DocumentContext.Provider>
  )
}
