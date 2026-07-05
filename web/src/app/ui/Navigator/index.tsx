import Link from "next/link";
import React, {useEffect, useState} from "react"
import {Folder, FolderOpen, InsertDriveFile, InsertDriveFileOutlined} from "@mui/icons-material";
import {NavigatorLink} from "@/lib/navigator"
import clsx from "clsx";
import {GetVehicle, Vehicle} from "@/lib/api";
import Image from "next/image";
import {MarqueBadge} from "./MarqueBadge";
import {LibrettoMark} from "@/app/ui/LibrettoMark";

const key = "navigator_position"
const navigatorDiv = "navigator"

function RootHeader() {
  return (
    <Link href="/" className={clsx("block bg-ink px-6 py-6 md:py-8 text-center border-b border-white/10")}>
      <div className={clsx("flex justify-center text-white")}>
        <LibrettoMark className={clsx("h-9 w-9 md:h-11 md:w-11")}/>
      </div>
      <p className={clsx("mt-2.5 font-mono text-base md:text-lg uppercase tracking-[0.32em] text-white")}>Libretto</p>
      <p className={clsx("mt-1 text-[11px] text-white/45")}>Service reference</p>
    </Link>
  )
}

function Header(params: { vehicle: string, year: number }) {
  const [vehicle, setVehicle] = useState<Vehicle>()

  useEffect(() => {
    GetVehicle(params.vehicle, params.year).then((data) => setVehicle(data));
  }, [params.vehicle, params.year])

  if (!vehicle) return (
    <div className={clsx("bg-ink px-4 py-3 md:px-6 md:py-6 border-b border-white/10")}>
      <div className={clsx("flex items-center gap-3 md:block")}>
        <div className={clsx("h-11 w-16 shrink-0 animate-pulse rounded bg-white/10 md:mx-auto md:h-24 md:w-2/3")}/>
        <div className={clsx("h-4 w-32 animate-pulse rounded bg-white/10 md:mx-auto md:mt-4 md:w-2/3")}/>
      </div>
    </div>
  )

  return (
    <Link href="/" className={clsx("block bg-ink px-4 py-3 md:px-6 md:py-6 border-b border-white/10")}>
      <div className={clsx("flex items-center gap-3 md:block")}>
        {/* Fixed-height box (matches the skeleton) so every vehicle photo — whatever
            its aspect ratio — occupies the same height and the tree below never jumps. */}
        <div className={clsx("shrink-0 md:mx-auto md:flex md:h-24 md:w-full md:items-center md:justify-center")}>
          <Image
            className={clsx("h-11 w-auto md:h-full md:w-auto md:max-w-full md:object-contain")}
            src={vehicle.image_url}
            width={500}
            height={500}
            alt={vehicle.name}
          />
        </div>
        <div className={clsx("min-w-0 md:mt-3 md:text-center")}>
          <p className={clsx("truncate text-sm font-semibold text-white md:text-base")}>{vehicle.name}</p>
          <p className={clsx("font-mono text-[10px] uppercase tracking-[0.18em] text-brass md:mt-1 md:text-[11px]")}>
            {params.vehicle} · {params.year}
          </p>
        </div>
      </div>
    </Link>
  )
}

function NavLinks(params: Params) {
  useEffect(() => {
    const value = localStorage.getItem(key) || ""
    const d = document.querySelector("#" + navigatorDiv)
    if (d) {
      d.scrollTop = Number(value)
    }
  }, [params])


  if (!params.navLinks) return <div id={navigatorDiv} className={clsx("w-full h-full overflow-y-auto")}/>

  return (
    <div id={navigatorDiv} className={clsx("w-full h-full overflow-y-auto py-1")}>
      {
        params.navLinks.map((link: NavigatorLink, index) => {
          // The current folder is a static "you are here" label — not a link — so it
          // no longer doubles (confusingly) as the go-up button. Going up gets its own
          // explicit control: a "Back" row up one level (the parent), and the top
          // "All vehicles" bar for the root, where there's no parent folder.
          if (link.kind === "open_folder") {
            const linkedVisualization = link.text.split(" ")[0]
            return (
              <div
                key={link.text}
                id={linkedVisualization}
                className={clsx(
                  "flex items-center gap-2.5 border-l-2 border-transparent px-4 py-2",
                  "mb-1 border-b border-white/10 text-[13px] font-semibold text-white",
                )}
              >
                <FolderOpen className={clsx("size-[18px] shrink-0 text-brass")}/>
                <p className={clsx("my-auto")}>{link.text}</p>
              </div>
            )
          }

          const isVehicle = link.kind === "vehicle"
          // Children of the current folder are indented one level beneath it, so the
          // tree structure reads visually.
          const isChild = link.kind === "folder" || link.kind === "drive_file"
          const iconColor = link.selected ? "text-brass" : "text-white/45"
          const className = clsx(
            "flex items-center border-l-2 text-[13px] transition-colors",
            isVehicle && "gap-3 px-4 py-2.5",
            isChild && "gap-2.5 py-2 pl-10 pr-4",
            link.selected
              ? "border-brass bg-white/[0.06] text-white font-medium"
              : "border-transparent text-white/70 hover:bg-white/[0.05] hover:text-white",
          )

          const linkedVisualization = link.text.split(" ")[0]

          return (
            <Link
              href={link.href}
              key={link.text}
              className={className}
              id={linkedVisualization}
              onClick={link.kind == "drive_file" ? saveToLocalStorage : resetLocalStorage}
            >
              {link.kind == "vehicle" && <MarqueBadge marque={link.icon}/>}
              {link.kind == "folder" && <Folder className={clsx("size-[18px] shrink-0", iconColor)}/>}
              {link.kind == "drive_file" && !link.selected && <InsertDriveFile className={clsx("size-[18px] shrink-0", iconColor)}/>}
              {link.kind == "drive_file" && link.selected &&
                  <InsertDriveFileOutlined className={clsx("size-[18px] shrink-0", iconColor)}/>}
              <p className={clsx("my-auto")}>{link.text}</p>
            </Link>
          );
        })
      }
    </div>
  );
}

export type Params = {
  navLinks: NavigatorLink[],
  location: number | null,
  vehicle: string | null,
  year: number | null,
}

const resetLocalStorage = function () {
  localStorage.setItem(key, "0");
}

const saveToLocalStorage = function () {
  const d = document.querySelector(`#${navigatorDiv}`)
  if (d) {
    localStorage.setItem(key, d.scrollTop.toLocaleString())
  }
}

export default function Index({navLinks, location, vehicle, year}: Params) {
  const isVehicleAndYearPresent = vehicle && year;
  const HeaderComponent = isVehicleAndYearPresent ? <Header vehicle={vehicle} year={year}/> : <RootHeader/>;

  return <div className={clsx("w-full h-full flex flex-col bg-ink border border-ink-2 overflow-hidden")}>
    {HeaderComponent}
    <NavLinks navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
  </div>
}
