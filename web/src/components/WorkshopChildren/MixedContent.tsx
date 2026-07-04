import {ContentMixed} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";
import React from "react";

export function MixedContent({obj: {inputs: {children}}, translations}: {
  obj: ContentMixed,
  translations: Map<string, string>
}) {

  const nested = Array.isArray(children) && children.map((c, index) => {
    return <WorkshopChildren obj={c} key={index} translations={translations}/>;
  });

  // TODO this can be a dropdown table or something

  return <mixed_component>
    <div _content-mixed="">
      {nested && nested}
    </div>
  </mixed_component>
}

