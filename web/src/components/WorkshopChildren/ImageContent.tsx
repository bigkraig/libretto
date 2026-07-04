import {ContentImage} from "@/lib/api/types";
import Image from "next/image";
import {GetWorkshopImageUrl} from "@/lib/api";
import React from "react";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";

export function ImageContent({
                               obj: {
                                 inputs: {
                                   id,
                                   mediacloudSmall,
                                   mediacloudNormal,
                                   mediacloudLarge,
                                   format,
                                   key,
                                   title,
                                   inTable,
                                   legend
                                 }
                               }, translations
                             }: { obj: ContentImage, translations: Map<string, string> }) {

  let legendElement;
  if (legend) {
    legendElement = <div _content-image="" className={"legend"}>
      <div _content-image="" className={"legend-table"}>
        {
          legend.map(([label, content], index) => {
            return <div _content-image="" key={index} className={"legend-item"}>
              <span _content-image="" className={"legend-cell"}>
                <WorkshopChildren obj={label} translations={translations}/>
              </span>
              <div _content-image="" className={"legend-cell"}>
                <WorkshopChildren obj={content} translations={translations}/>
              </div>
            </div>
          })}
      </div>
    </div>;
  }

  return <image_component _content-image="">
    <div _content-image="" className={"image-container"}>
      <div _content-image="" className={"image-preview"}>
        <Image _content-image="" id={id} src={GetWorkshopImageUrl(key, "large")}
               alt={"picture-with-legend"} width={inTable?180:640} height={0}/>
        <span _content-image=""></span>
      </div>
      <div _content-image="" className={"image-title"}>{title}</div>
      {legendElement && legendElement}
    </div>
  </image_component>
}