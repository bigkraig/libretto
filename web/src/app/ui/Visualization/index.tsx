import React, {useEffect, useMemo, useRef} from "react";
import {useRouter} from 'next/navigation'
import clsx from "clsx";
import useSWR from "swr";
import {GetTreeNodes} from "@/lib/api/GetTreeNodes";
import {GetIllustration} from "@/lib/api";
import {NavigatorLink} from "@/lib/navigator";

function NoVisualization({hasVehicle}: { hasVehicle: boolean }) {
  return (
    <div className={clsx("m-auto flex flex-col items-center gap-2 px-6 text-center")}>
      <svg viewBox="0 0 24 24" className={clsx("size-9 text-line")} fill="none" stroke="currentColor" strokeWidth="1.5">
        <rect x="3" y="4" width="18" height="14" rx="1.5"/>
        <path d="M3 15l4.5-4.5 3 3L15 9l6 6"/>
        <circle cx="8.5" cy="8.5" r="1.25"/>
      </svg>
      <div className={clsx("text-sm text-ink")}>{hasVehicle ? "No diagram for this component" : "Select a vehicle"}</div>
      <div className={clsx("max-w-xs text-xs text-muted")}>
        {hasVehicle ? "Open a document below to view it." : "Choose a vehicle from the list to get started."}
      </div>
    </div>
  )
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

export type Params = {
  vehicle: string | null,
  year: number | null,
  location: number | null,
  navLinks: NavigatorLink[],
}

export default function Index({vehicle, year, location, navLinks}: Params) {
  const router = useRouter()
  const containerRef = useRef<HTMLDivElement>(null)
  const tooltipRef = useRef<HTMLDivElement>(null)
  const hasVehicle = !!(vehicle && year)

  // Illustration for the current node, cached by (vehicle, year, location). Throws
  // when there's no diagram, which SWR surfaces as `error`.
  const {data, error} = useSWR<SVGData>(
    hasVehicle ? ["illustration", vehicle, year, location] : null,
    () => retrieveSVG(vehicle!, year!, location),
  )

  // Hotspot id -> destination href / tooltip label, from the current nav links.
  const {hrefById, labelById} = useMemo(() => {
    const hrefById = new Map<string, string>()
    const labelById = new Map<string, string>()
    for (const l of navLinks) {
      const key = (l.kind == "drive_file" ? "FES_" : "NAV_") + l.location
      hrefById.set(key, l.href)
      labelById.set(key, l.text)
    }
    return {hrefById, labelById}
  }, [navLinks])

  // The illustration ships as an <svg> with fixed width/height. After React injects
  // it, size it to a consistent box, tag the hotspots selectable/not, and highlight
  // the active one. React treats the innerHTML as opaque, so these imperative tweaks
  // survive re-renders (they only re-run when the illustration or links change).
  useEffect(() => {
    const container = containerRef.current
    if (!container || !data) return

    const inner = container.querySelector('svg') as SVGSVGElement | null
    if (!inner) return

    if (!inner.getAttribute('viewBox')) {
      const w = parseFloat(inner.getAttribute('width') || '')
      const h = parseFloat(inner.getAttribute('height') || '')
      if (w && h) inner.setAttribute('viewBox', `0 0 ${w} ${h}`)
    }
    inner.removeAttribute('width')
    inner.removeAttribute('height')
    inner.setAttribute('preserveAspectRatio', 'xMidYMin meet')
    inner.style.width = '100%'
    inner.style.height = 'auto'
    // Cap by viewport height so a landscape illustration can't grow taller than the
    // pane and overflow; meet then letterboxes the content, so nothing is clipped.
    inner.style.maxHeight = '58vh'

    for (const hotspot of container.querySelectorAll('#CC_HOTSPOT')) {
      for (const tag of ["path", "polygon", "rect"]) {
        for (const poly of hotspot.getElementsByTagName(tag) as HTMLCollectionOf<SVGElement>) {
          poly.classList.add(hrefById.has(poly.id) ? 'selectable' : 'not-selectable')
        }
      }
    }

    for (const tag of ["path", "polygon"]) {
      for (const poly of inner.getElementsByTagName(tag) as HTMLCollectionOf<SVGElement>) {
        if (poly.id.endsWith(`_${data.active_location}`)) {
          poly.style.fill = '#C0862C'
          poly.style.fillOpacity = "0.55"
        }
      }
    }
  }, [data, hrefById])

  // Walk up from the event target to the first hotspot element we know about.
  function hotspotIdAt(target: EventTarget | null): string | null {
    let el = target as Element | null
    while (el && el !== containerRef.current) {
      if (el.id && hrefById.has(el.id)) return el.id
      el = el.parentElement
    }
    return null
  }

  function onClick(e: React.MouseEvent) {
    const id = hotspotIdAt(e.target)
    const href = id && hrefById.get(id)
    if (href) router.push(href)
  }

  function onMouseMove(e: React.MouseEvent) {
    const tooltip = tooltipRef.current
    if (!tooltip) return
    const id = hotspotIdAt(e.target)
    if (!id) { tooltip.style.display = 'none'; return }
    tooltip.textContent = labelById.get(id) ?? "No tooltip available"
    tooltip.style.display = 'block'
    tooltip.style.left = `${e.clientX - 20}px`
    tooltip.style.top = `${e.clientY + 20}px`
  }

  function onMouseLeave() {
    if (tooltipRef.current) tooltipRef.current.style.display = 'none'
  }

  let content: React.ReactNode
  if (!hasVehicle) {
    content = <NoVisualization hasVehicle={false}/>
  } else if (error) {
    content = <NoVisualization hasVehicle={true}/>
  } else if (!data) {
    content = <div className={clsx("m-auto")}/>
  } else {
    content = (
      <div
        ref={containerRef}
        className={clsx("flex h-full w-full items-start justify-center overflow-hidden")}
        onClick={onClick}
        onMouseMove={onMouseMove}
        onMouseLeave={onMouseLeave}
        dangerouslySetInnerHTML={{__html: data.content}}
      />
    )
  }

  return (
    <div className={clsx("w-full h-full p-2 bg-white")}>
      <div ref={tooltipRef}
           className={clsx("hidden fixed z-30 rounded-sm bg-ink px-2 py-1 text-xs text-white pointer-events-none")}/>
      <div className={clsx("flex h-full w-full items-start justify-center overflow-hidden")}>
        {content}
      </div>
    </div>
  );
}
