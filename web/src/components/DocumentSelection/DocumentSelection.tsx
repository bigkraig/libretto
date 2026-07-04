import React, {useEffect, useState} from "react";
import {usePathname} from "next/navigation";
import {IDocument} from "@/lib/api";
import {SearchDocuments} from "@/lib/api/SearchDocuments";
import clsx from "clsx";
import {Listbox, ListboxButton, ListboxOption, ListboxOptions} from "@headlessui/react";
import {ExpandMore} from "@mui/icons-material";
import {WorkshopLiterature} from "@/components/WorkshopLiterature";

export function DocumentSelection({documentType, aposNumber, inline, translations}: {
  documentType: string,
  aposNumber: string,
  inline: boolean
  translations: Map<string, string>
}) {
  let parts = usePathname().split("?")[0].split("/");
  let year = Number(parts[2]);
  let vehicle = parts[3];

  console.log("DocumentSelection", year, vehicle, aposNumber)

  const [documents, setDocuments] = useState<IDocument[]>([])
  const [selectedDocument, setSelectedDocument] = useState<IDocument>()
  const [inlineDocument, setInlineDocument] = useState<string | undefined>(undefined)

  useEffect(() => {
    if (!vehicle || !year) {
      return
    }

    SearchDocuments(year, vehicle, aposNumber).then(docs => setDocuments(docs));
  }, [vehicle, year])

  if (documents.length == 0) {
    return <></>
  }

  if (documents.length == 1) {
    if (!inline) {
      const link = `/documents/${year}/${vehicle}/${documents[0].hkap_id}`;
      window.open(link, "_self");
      return;
    } else {
      return <div _content-nested="" className={"border-l-4 pl-4"}>
        <WorkshopLiterature hkap_id={documents[0].hkap_id} translations={translations}/>
      </div>
    }
  }

  return <div className='documentSelector'>
    <div className={clsx("my-2 ml-10 z-20 w-[800px]")}>
      <div className={clsx('outline outline-1')}>
        <p className={clsx('z-10 px-1 mt-[-11px] ml-5 bg-zinc-50 absolute place-self-start')}>Target document</p>
        <div className={clsx("flex flex-row justify-end min-w-full")}>
          <Listbox value={selectedDocument}
                   onChange={function (value: IDocument | undefined) {
                     if (!value) {
                       return
                     }
                     setSelectedDocument(value);

                     if (!inline) {
                       const link = `/documents/${year}/${vehicle}/${value.hkap_id}`;
                       window.open(link, "_self");
                     } else {
                       setInlineDocument(value.hkap_id);
                       // alert(`should show ${documents[0].hkap_id}`)
                       // return <div _content-nested="" className={"border-l-4 pl-4"}>
                       //   <WorkshopLiterature hkap_id={Number(documents[0].hkap_id)} translations={translations}/>
                       // </div>
                     }
                   }}>
            <ListboxButton className={clsx('flex bg-zinc-100 min-w-full')}>
              <p
                // https://tailwindcss.com/docs/hover-focus-and-other-states#placeholder-text
                className={clsx('px-4 py-2.5 p-1.5 pr-4 m-auto text-left bg-zinc-50 w-full ',
                  selectedDocument?.title ? "text-zinc-900" : "text-zinc-400")}>
                {selectedDocument?.title ? `${selectedDocument.vehicle_component} ${selectedDocument?.title}` : "Please select a target document"}
              </p>
              <div className={clsx('bg-zinc-700 text-white h-full flex aspect-square')}>
                <ExpandMore className={clsx('m-auto text-xl')}/>
              </div>
            </ListboxButton>
            <ListboxOptions anchor="bottom" transition
                            className={clsx('bg-zinc-50 outline outline-1 outline-black w-[var(--button-width)]')}>
              {documents.map((f) => {
                return (
                  <ListboxOption key={f.title} value={f}
                                 className="group flex cursor-default
                            items-center gap-2 py-1.5 px-3 min-w-full
                            select-none data-[focus]:bg-zinc-700 data-[focus]:text-white">
                    {f.vehicle_component} {f.title}
                  </ListboxOption>
                )
              })}
            </ListboxOptions>
          </Listbox>
        </div>
      </div>
    </div>
    {
      inlineDocument &&
        <div _content-nested="" className={"border-l-4 pl-4"}>
            <WorkshopLiterature hkap_id={inlineDocument} translations={translations}/>
        </div>
    }
  </div>
}