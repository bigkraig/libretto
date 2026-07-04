import {ContentParagraph} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";

export function ParagraphContent({obj: {inputs: {children}}, translations}: {
  obj: ContentParagraph,
  translations: Map<string, string>
}) {
  return <paragraph_component _content-paragraph="">
    <div _content-paragraph="">
      <WorkshopChildren obj={children} translations={translations}/>
    </div>
  </paragraph_component>
}

