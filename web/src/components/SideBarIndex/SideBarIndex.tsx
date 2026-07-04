import React from "react";
import clsx from "clsx";
import {Toc} from "@/lib/api/types";
import {useRouter} from "next/navigation";
import {PSArrowRight} from "@/lib/icons";

export function SideBarIndex({toc}: { toc: Toc[] | undefined }) {
  const router = useRouter();

  if (!toc) {
    return <></>
  }

  return <>
    <div className={clsx("mb-2")}>
      <div className={clsx("p-8")}></div>
      <span className={clsx("font-bold pl-6 text-porschegrey")}>Index</span>
    </div>
    <div className={clsx("border-b-2 ml-2 mr-2")}></div>
    <div className={clsx("mt-4")}>
      {
        toc.map(function (entry) {
          return <div key={entry.anchor}>
            <button className={clsx("flex flex-row px-2 pt-1 my-auto items-center")} id={entry.anchor}
                     onClick={function () {
              router.push("#" + entry.anchor)
            }}>
              <PSArrowRight contentType={"_sidebar"} size="text-2xl" color="text-porschered"/>
              <p className={clsx("pl-1 text-left align-middle leading-tight")}>{entry.label}</p>
            </button>
          </div>
        })
      }
    </div>
  </>
}