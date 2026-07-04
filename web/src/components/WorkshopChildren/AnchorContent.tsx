import {ContentAnchor} from "@/lib/api/types";

export function AnchorContent({obj: {inputs: {id}}}: { obj: ContentAnchor, translations: Map<string, string> }) {
  return <anchor_component _content-anchor="">
    <a _content-anchor="" id={id}/>
  </anchor_component>
}