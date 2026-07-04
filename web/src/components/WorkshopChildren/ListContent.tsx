import {ContentList} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";
import React from "react";

export function ListContent({obj: {inputs: {items}}, translations}: {
  obj: ContentList,
  translations: Map<string, string>
}) {

  const nested = Array.isArray(items) && items.map((c, index) => {
    return <li _content-list="" key={index}><WorkshopChildren obj={c} key={index} translations={translations}/></li>;
  });

  return <list_component _content-list="">
    <div _content-list="" className={"mb-[5px]"}>
      <ul _content-list="" className={"text-porschegrey list-disc list-inside"}>
        {nested && nested}
      </ul>
    </div>
  </list_component>
}
