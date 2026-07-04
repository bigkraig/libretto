import React, {useEffect, useState} from "react";
import {GetWorkshopLiterature, IWorkshopLiterature} from "@/lib/api";
import {ToolsTable} from "@/app/ui/ToolsTable";
import {TechnicalValuesTable} from "@/app/ui/TechnicalValuesTable";
import {WorkshopChildren} from "@/components/WorkshopChildren";
import clsx from "clsx";
import {FileFormat} from "@/lib/api/types";
import {PDF} from "@/components/PDF";
import {PartsTable} from "@/app/ui/PartsTable";

export function WorkshopLiterature({hkap_id, translations}: { hkap_id: string, translations: Map<string, string> }) {
  let [workshopLiterature, setWorkshopLiterature] = useState<IWorkshopLiterature>()
  useEffect(() => {
    GetWorkshopLiterature(hkap_id).then(content => setWorkshopLiterature(content));
  }, [hkap_id])

  if (!workshopLiterature) {
    return <div></div>
  }

  return <div className={clsx("document-container", "mx-0 sm:mx-4", "bg-paper")}>
    {/*TEQ, 2602 from taycan*/}
    {/*<LaborOpsTable laborOps={data.laborops}/>*/}
    <PartsTable parts={workshopLiterature.parts} translations={translations}/>
    <ToolsTable tools={workshopLiterature.tools} translations={translations}/>
    <TechnicalValuesTable values={workshopLiterature.techvalues} translations={translations}/>
    <div className={clsx("mt-4")}/>
    {
      workshopLiterature.fileFormat == FileFormat.XML &&
        <WorkshopChildren obj={workshopLiterature.content} translations={translations}/>
    }
    {
      workshopLiterature.fileFormat == FileFormat.PDF &&
        <PDF id={workshopLiterature.mediaCloudFileId}/>
    }
  </div>
}