import {ContentStatic} from "@/lib/api/types";
import React from "react";

export function StaticContent({obj: {inputs: {html, para, translateIds}}, translations}: {
  obj: ContentStatic,
  translations: Map<string, string>
}) {
  if (translateIds) {
    for (const id of translateIds) {
      html = html?.replaceAll('$' + id + '$', translations.get(id) ?? id)
      if (!translations.get(id)) {
        alert(`Missing translation for ${id}`);
      }
    }
  }

  html = html?.replace(/href="" target="([^"]*)"/g, 'href="#" onclick="location.replace(\'#$1\'); return false;"');

  let component = para ? <p _content-static="" dangerouslySetInnerHTML={{__html: html ?? ""}}/> :
    <span _content-static="" dangerouslySetInnerHTML={{__html: html ?? ""}}/>;

  return <static_component _content-static="">
    {component}
  </static_component>
}

