import React, {useEffect, useState} from "react";
import {useRouter} from 'next/navigation'
import {AppRouterInstance} from "next/dist/shared/lib/app-router-context.shared-runtime";
import clsx from "clsx";
import {GetTreeNodes} from "@/lib/api/GetTreeNodes";
import {GetIllustration} from "@/lib/api";
import {NavigatorLink} from "@/lib/navigator";

function NoVisualization() {
  return <div className={clsx("m-auto")}>No visualization available</div>
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
  flex.className = "font-bold flex h-full outline outline-1 outline-zinc-300 aspect-[1.41] text-center";
  var svg = document.createElement('div');
  svg.innerHTML = "No visualization available";
  svg.className = "m-auto";
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
  var svg = document.createElement('svg');
  svg.id = "navigatorImage"
  svg.innerHTML = data.content;
  svg.className = "h-full outline outline-1 outline-zinc-300 aspect-[1.41]";
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
    <div className={clsx("w-full h-full pl-2 pb-2 justify-items-start bg-white")}>
      <div id="tooltip" className={clsx("hidden absolute bg-porschegrey text-white px-[10px] py-[5px]")}></div>
      <div id="navigatorImage"
           className={clsx("font-bold flex h-full outline outline-1 outline-zinc-300 aspect-[1.41] text-center")}>
        <SVG vehicle={vehicle} year={year} location={location} navLinks={navLinks}/>
      </div>
    </div>
  );
}