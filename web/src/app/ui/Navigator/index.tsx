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
    <Link href="/" className={clsx("block bg-ink px-6 py-6 md:py-8 text-center")}>
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
    <div className={clsx("bg-ink px-4 py-3 md:px-6 md:py-6")}>
      <div className={clsx("flex items-center gap-3 md:block")}>
        <div className={clsx("h-11 w-16 shrink-0 animate-pulse rounded bg-white/10 md:mx-auto md:h-auto md:aspect-[3/2] md:w-full md:max-w-[220px]")}/>
        <div className={clsx("h-4 w-32 animate-pulse rounded bg-white/10 md:mx-auto md:mt-4 md:w-2/3")}/>
      </div>
    </div>
  )

  return (
    <Link href="/" className={clsx("block bg-ink px-4 py-3 md:px-6 md:py-6")}>
      <div className={clsx("flex items-center gap-3 md:block")}>
        <Image
          className={clsx("h-11 w-auto shrink-0 md:mx-auto md:h-auto md:w-full md:max-w-[220px]")}
          src={vehicle.image_url}
          width={500}
          height={500}
          alt={vehicle.name}
        />
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
          const isOpen = link.kind === "open_folder"
          const isVehicle = link.kind === "vehicle"
          const iconColor = link.selected ? "text-brass" : "text-muted"
          const className = clsx(
            "flex items-center border-l-2 text-[13px] transition-colors",
            isVehicle ? "gap-3 px-4 py-2.5" : "gap-2.5 px-4 py-2",
            link.selected
              ? "border-brass bg-brass-wash text-ink font-medium"
              : "border-transparent text-ink/85 hover:bg-brass-wash hover:text-ink",
            isOpen && "font-semibold text-ink",
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
              {link.kind == "open_folder" && <FolderOpen className={clsx("size-[18px] shrink-0 text-brass")}/>}
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

  return <div className={clsx("w-full h-full flex flex-col bg-white border-r border-line")}>
    {HeaderComponent}
    <NavLinks navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
  </div>
}
