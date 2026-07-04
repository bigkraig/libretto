import React, {useEffect, useState} from "react";
import {useRouter} from 'next/navigation'
import {AppRouterInstance} from "next/dist/shared/lib/app-router-context.shared-runtime";
import clsx from "clsx";
import {GetTreeNodes} from "@/lib/api/GetTreeNodes";
import {GetIllustration} from "@/lib/api";
import {NavigatorLink} from "@/lib/navigator";

function NoVisualization() {
  return (
    <div className={clsx("m-auto flex flex-col items-center gap-2 px-6 text-center")}>
      <svg viewBox="0 0 24 24" className={clsx("size-9 text-line")} fill="none" stroke="currentColor" strokeWidth="1.5">
        <rect x="3" y="4" width="18" height="14" rx="1.5"/>
        <path d="M3 15l4.5-4.5 3 3L15 9l6 6"/>
        <circle cx="8.5" cy="8.5" r="1.25"/>
      </svg>
      <div className={clsx("text-sm text-ink")}>No diagram for this component</div>
      <div className={clsx("max-w-xs text-xs text-muted")}>Choose a component from the tree to view its exploded diagram.</div>
    </div>
  )
}

function showTooltip(evt: MouseEvent, text: string | undefined) {
  let tooltip = document.getElementById("tooltip") as HTMLElement;
  tooltip.innerHTML = text?? "No tooltip available";
  tooltip.style.display = "block";
  tooltip.style.left = evt.pageX + -20 + 'px';
  tooltip.style.top = evt.pageY + 20 + 'px';
}

function hideTooltip() {
  var tooltip = document.getElementById("tooltip") as HTMLElement;
  tooltip.style.display = "none";
}

function resetSVG() {
  var flex = document.createElement('div')
  flex.id = "navigatorImage"
  flex.className = "flex h-full w-full items-center justify-center bg-white text-center";
  var svg = document.createElement('div');
  svg.innerHTML =
    '<div style="font-size:13px;color:#16181D">No diagram for this component</div>' +
    '<div style="font-size:12px;color:#6B6E73;margin-top:3px">Open a document below to view it.</div>';
  svg.className = "m-auto px-6";
  flex.appendChild(svg)
  const navigatorImage = document.getElementById('navigatorImage');
  navigatorImage?.parentNode?.replaceChild(flex, navigatorImage);
}

function smashSVG(router: AppRouterInstance, navLinks: NavigatorLink[], data: SVGData) {
  if (typeof document == 'undefined') {
    return
  }

  const img = document.getElementById('navigatorImage')
  if (!img) {
    return
  }

  const navigatorImage = document.getElementById('navigatorImage');
  const svg = document.createElement('div');
  svg.id = "navigatorImage";
  svg.className = "flex h-full w-full items-center justify-center overflow-hidden";
  svg.innerHTML = data.content;
  // The illustration ships with fixed width/height and would overflow. Make it
  // scale to fit the pane while keeping its aspect ratio.
  const inner = svg.querySelector('svg') as SVGSVGElement | null;
  if (inner) {
    if (!inner.getAttribute('viewBox')) {
      const w = parseFloat(inner.getAttribute('width') || '');
      const h = parseFloat(inner.getAttribute('height') || '');
      if (w && h) inner.setAttribute('viewBox', `0 0 ${w} ${h}`);
    }
    inner.removeAttribute('width');
    inner.removeAttribute('height');
    inner.setAttribute('preserveAspectRatio', 'xMidYMid meet');
    inner.style.width = '100%';
    inner.style.height = '100%';
    inner.style.maxWidth = '100%';
    inner.style.maxHeight = '100%';
  }
  navigatorImage?.parentNode?.replaceChild(svg, navigatorImage);

  const navigatorLocations = new Map(navLinks.map(l => [
    (l.kind == "drive_file" ? "FES_" : "NAV_") + l.location, l.href]));

  const navToolTips = new Map(navLinks.map(l => [
    (l.kind == "drive_file" ? "FES_" : "NAV_") + l.location, l.text]));

  for (const hotspot of svg.querySelectorAll('#CC_HOTSPOT')) {
    for (const qualifiedName of ["path", "polygon", "rect"]) {
      for (const polygon of hotspot.getElementsByTagName(qualifiedName) as HTMLCollectionOf<SVGElement>) {

        // If the navigator has a location that matches the polygon id, we make it selectable
        if (navigatorLocations.has(polygon.id)) {
          polygon.setAttribute('class', polygon.getAttribute('class') + ' selectable')
          polygon.addEventListener('mousemove', (evt) => showTooltip(evt, navToolTips.get(polygon.id)));
          polygon.addEventListener('mouseout', () => hideTooltip());
        } else {
          polygon.setAttribute('class', polygon.getAttribute('class') + ' not-selectable')
        }

        polygon.onclick = () => {
          const href = navigatorLocations.get(polygon.id);
          if (href) {
            router.push(href)
          }
        }
      }
    }
  }

  // Highlights the active polygon
  for (const qualifiedName of ["path", "polygon"]) {
    const polygons = svg.getElementsByTagName(qualifiedName) as HTMLCollectionOf<SVGElement>;
    for (const polygon of polygons) {
      if (polygon.id.endsWith(`_${data.active_location}`)) {
        polygon.style.fill = '#690';
        polygon.style.fillOpacity = "0.8"
      }
    }
  }
  return
}

async function retrieveSVG(vehicle: string, year: number, location: number | null): Promise<SVGData> {
  let node = await GetTreeNodes(vehicle, year, location);
  let active_location = node.location;
  if (node.isDriveFile()) {
    node = await GetTreeNodes(vehicle, year, node.parent_node_id);
  }

  if (node.illustration_id == 0) {
    throw new Error("Invalid illustration_id=0")
  }

  return {active_location: active_location, content: await GetIllustration(node.illustration_id)};
}

interface SVGData {
  active_location: string,
  content: string,
}

function SVG(params: Params) {
  const router = useRouter()
  let [svg, setSvg] = useState<SVGData>()

  useEffect(() => {
    if (!params.vehicle || !params.year) {
      setSvg(undefined);
      resetSVG();
      return
    }

    retrieveSVG(params.vehicle, params.year, params.location).then((data) => setSvg(data)).catch(e => {
      console.log("i got an error", e)
      // Case where the illustration_id is no good
      setSvg(undefined);
      resetSVG();
    })
    if (svg) {
      smashSVG(router, params.navLinks, svg);
    }
  }, [router, params, svg?.active_location])

  if (!svg) return NoVisualization()
}


export type Params = {
  vehicle: string | null,
  year: number | null,
  location: number | null,
  navLinks: NavigatorLink[],
}

export default function Index({vehicle, year, location, navLinks}: Params) {
  return (
    <div className={clsx("w-full h-full p-2 bg-white")}>
      <div id="tooltip" className={clsx("hidden absolute z-30 rounded-sm bg-ink px-2 py-1 text-xs text-white pointer-events-none")}></div>
      <div id="navigatorImage"
           className={clsx("flex h-full w-full items-center justify-center overflow-hidden")}>
        <SVG vehicle={vehicle} year={year} location={location} navLinks={navLinks}/>
      </div>
    </div>
  );
}