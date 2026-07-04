import {Content, ContentLink} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";
import React, {useContext} from "react";
import {PSArrowRight} from "@/lib/icons";
import {DocumentContext, waitVisible} from "@/lib";

// TODO it would be better to add click handlers to all a.qv, but react sucks
export function clickHandler(t: any) {
  // t.preventDefault();
  // const o = this.location.path();
  // else if (n.classList.contains("qv-vws-id")) this.router.navigate([d.p.NON_COMPONENT_SEARCH.ROUTE], {
  //   queryParams: {
  //     types: M.WD,
  //     [d.p.QUERY_PARAMS.BACK_URL]: o
  //   }
  // }); else if (n.classList.contains("qv-extern")) {
  //   const s = n.target;
  //   M[s] ? this.router.navigate([d.p.NON_COMPONENT_SEARCH.ROUTE], {queryParams: {types: s}}) : this.router.navigate([d.p.COMPONENT_SEARCH.ROUTE], {
  //     queryParams: {docTypes: s},
  //     queryParamsHandling: "merge"
  //   })
  // } else n.classList.contains("qv") && this.navigateToFragmentWithTimeout(n.target)
}

function extractLink(html: string, table: string, tableVisible: boolean, toggleTable: any) {
  const toggle = function (event: any) {
    event.preventDefault();
    if (!tableVisible) {
      toggleTable();
    }
    let href = event.target.getAttribute('href');
    waitVisible(document.getElementById(table), () => location.replace(href));
  }
  var el = document.createElement('html');
  el.innerHTML = html;
  const a = el.getElementsByTagName('a')[0];
  if (a.target != "") {
    return <a href={"#" + a.target} onClick={toggle}>{a.text}</a>;
  } else {
    return <a href={a.href} onClick={toggle}>{a.text}</a>;
  }
}

export function LinkContent({obj: {inputs: {html, id, children}}, translations}: {
  obj: ContentLink,
  translations: Map<string, string>
}) {
  const {
    isToolsTableVisible,
    toggleToolsTableVisible,
    isTechValuesTableVisible,
    toggleTechValuesTableVisible,
    isPartsTableVisible,
    togglePartsTableVisible,
  } = useContext(
    DocumentContext
  );

  if (children && Array.isArray(children)) {
    if (children.length != 1) {
      alert(`Got some kind of invalid link content ${html} ${id}, ${children}`);
      return <></>
    }
  }


  let link;

  if (html?.includes('href class="qv qv-tool" target="')) {
    link = extractLink(html, "tools-table", isToolsTableVisible, toggleToolsTableVisible);
  }

  if (html?.includes('class="qv qv-techvalue"')) {
    link = extractLink(html, "technical-values-table", isTechValuesTableVisible, toggleTechValuesTableVisible);
  }

  if (html?.includes('class="qv qv-ersatzteil"')) {
    link = extractLink(html, "parts-table", isPartsTableVisible, togglePartsTableVisible);
  }

  const child = children && Array.isArray(children) ? children[0] as Content : false;
  const nested = child ? <WorkshopChildren obj={child} translations={translations}/> : false;

  // console.log(nested);
  return <link_component _content-link="" is={" "}>
    <span _content-link="" className={"text-link"}>
      <PSArrowRight contentType="_content-link" size={"text-2xl"}/>
      <span _content-link="" className={"link"}>
        {link ? link : <span _content-static="" dangerouslySetInnerHTML={{__html: html ?? ""}}/>}
      </span>
  </span>
    {
      nested && nested
    }
  </link_component>
}
