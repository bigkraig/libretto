import React from "react";
import {
  Children,
  instanceOfContentAnchor,
  instanceOfContentImage,
  instanceOfContentLink,
  instanceOfContentList,
  instanceOfContentMixed,
  instanceOfContentNested,
  instanceOfContentParagraph,
  instanceOfContentPlot, instanceOfContentSection,
  instanceOfContentStatic, instanceOfContentTable,
  instanceOfContentWarning
} from "@/lib/api/types";
import {AnchorContent} from "@/components/WorkshopChildren/AnchorContent";
import {ParagraphContent} from "@/components/WorkshopChildren/ParagraphContent";
import {PlotContent} from "@/components/WorkshopChildren/PlotContent";
import {StaticContent} from "@/components/WorkshopChildren/StaticContent";
import {WarningContent} from "@/components/WorkshopChildren/WarningContent";
import clsx from "clsx";
import {ListContent} from "@/components/WorkshopChildren/ListContent";
import {MixedContent} from "@/components/WorkshopChildren/MixedContent";
import {LinkContent} from "@/components/WorkshopChildren/LinkContent";
import {ImageContent} from "@/components/WorkshopChildren/ImageContent";
import {NestedContent} from "@/components/WorkshopChildren/NestedContent";
import {SectionContent} from "@/components/WorkshopChildren/SectionContent";
import {TableContent} from "@/components/WorkshopChildren/TableContent";

export function WorkshopChildren({obj, translations}: { obj: Children | undefined, translations: Map<string, string> }) {
  if (!obj) {
    return <></>
  }

  // case where its a list of nodes, we go deeper into rabbit hole
  if (Array.isArray(obj)) {
    const nested = obj.map((c, index) => {
      return <WorkshopChildren obj={c} key={index} translations={translations}></WorkshopChildren>;
    });
    return <div className={clsx("display:block")}>{nested}</div>
  }

  // number ?
  if (typeof obj === "string") {
    return <span dangerouslySetInnerHTML={{__html: obj}}></span>;
  }

  if (typeof obj === "number") {
    return obj;
  }

  if (instanceOfContentAnchor(obj)) {
    return <AnchorContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentImage(obj)) {
    return <ImageContent obj={obj} translations={translations}/>
  }
  //   | { type: 'InvoiceTable'; value: ContentInvoiceTable }
  //   | { type: 'laborpos'; value: ContentLaborpos }
  if (instanceOfContentLink(obj)) {
    return <LinkContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentList(obj)) {
    return <ListContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentMixed(obj)) {
    return <MixedContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentNested(obj)) {
    return <NestedContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentList(obj)) {
    return <ListContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentParagraph(obj)) {
    return <ParagraphContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentPlot(obj)) {
    return <PlotContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentSection(obj)) {
    return <SectionContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentStatic(obj)) {
    return <StaticContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentTable(obj)) {
    return <TableContent obj={obj} translations={translations}/>
  }
  if (instanceOfContentWarning(obj)) {
    return <WarningContent obj={obj} translations={translations}/>
  }

  return <div><b>unhandled child {obj.type}</b></div>
}