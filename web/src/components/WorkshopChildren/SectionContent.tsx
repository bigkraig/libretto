import {ContentSection} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";


// TODO CSS

export function SectionContent({obj: {inputs: {title, titleUiText, children}}, translations}: {
  obj: ContentSection,
  translations: Map<string, string>
}) {
  if (titleUiText) {
    title = translations.get(titleUiText);
  }

  return <section_component _content-section="">
    <div _content-section="" className={"mt-[8px] flex flex-wrap"}>
      <div _content-section="" className="flex-none font-bold text-[14px] tracking-[0] text-[#494e51] w-[150px]">
        {title}
      </div>
      <div _content-section="" className="flex-none w-full w-[83.333333%]">
        {children && <WorkshopChildren obj={children} translations={translations}/>}
      </div>
    </div>
  </section_component>
}

