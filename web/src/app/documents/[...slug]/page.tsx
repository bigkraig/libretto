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

  if (!workshopLiterature) {
    return
  }

  if (!translations) {
    return
  }

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
      <main>
        <div className={clsx("grid h-screen w-full grid-cols-6 grid-rows-[7rem,100%] bg-zinc-100")}>
          <div id="navBar" className={clsx("row-span-1 col-span-5 bg-white inline-flex items-center")}>
            <div className={clsx("flex pt-3 pb-3 bg-white")}>
              <div
                className={clsx("p-1.5 bg-zinc-700 text-white my-auto hover:bg-red-700 transition-colors duration-300 ease-in-out cursor-pointer")}
                onClick={back}>
                <ArrowBack className={clsx('m-auto')}/>
              </div>
              <span
                className={clsx("text-xl font-bold my-auto pl-4")}>{workshopLiterature.documentType}, {workshopLiterature.kdnr} {workshopLiterature.title}</span>
            </div>
          </div>
          <div className={clsx("col-span-5 print:overflow-visible mb-4 m-2")}>
            <WorkshopLiterature hkap_id={hkap_id} translations={translations}/>
          </div>
          <div id="sideBar"
               className={clsx("w-full col-start-6 row-start-1 row-span-2 col-span-1 bg-white border-l print:hidden")}>
            <div className={clsx("flex flex-col fixed")}>
              <SideBarIndex toc={workshopLiterature.toc}></SideBarIndex>
            </div>
          </div>
        </div>
      </main>
    </DocumentContext.Provider>
  )
}
