import Link from "next/link";
import React, {useEffect, useState} from "react"
import {Folder, FolderOpen, InsertDriveFile, InsertDriveFileOutlined} from "@mui/icons-material";
import {PorscheAftersales} from "@/lib/fonts";
import {NavigatorLink} from "@/lib/navigator"
import clsx from "clsx";
import {GetVehicle, Vehicle} from "@/lib/api";
import Image from "next/image";
import {PSApplication} from "@/lib/icons";

const key = "navigator_position"
const navigatorDiv = "navigator"

function RootHeader() {
  return <Link className={clsx("rounded-t-lg bg-zinc-600 p-8")} href="/">
    <div className={clsx("text-white text-center text-8xl")}>
      <PSApplication contentType={"_content-link"} size={clsx("text-[64px]")} />
      <p className={clsx("text-xl font-bold")}>Libretto</p>
    </div>
  </Link>
}

function Header(params: { vehicle: string, year: number }) {
  const [vehicle, setVehicle] = useState<Vehicle>()

  useEffect(() => {
    GetVehicle(params.vehicle, params.year).then((data) => setVehicle(data));
  }, [params.vehicle, params.year])

  if (!vehicle) return <div className={clsx("rounded-t-lg bg-zinc-600 p-8 text-white text-center")}>Loading</div>

  return (
    <Link className={clsx("rounded-t-lg bg-zinc-600 p-8")} href="/">
      <Image
        src={vehicle.image_url}
        width={500}
        height={500}
        alt="Vehicle Image"
      />
      <p className={clsx("text-center pt-4 text-white text-xl font-bold")}>{vehicle.name}</p>
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


  if (!params.navLinks) return <div id={navigatorDiv} className={clsx("w-full h-full overflow-scroll divide-y")}/>

  return (
    <div id={navigatorDiv} className={clsx("w-full h-full overflow-scroll divide-y")}>
      {
        params.navLinks.map((link: NavigatorLink, index) => {
          let className = "flex gap-2 pt-2 pb-2 hover:bg-zinc-400 "

          if (link.selected) {
            className = className + " bg-zinc-400"
          }

          if (link.kind != "open_folder") {
            className = className + " px-5"
          } else {
            className = className + " font-bold"
          }

          const linkedVisualization = link.text.split(" ")[0]

          return (
            <Link
              href={link.href}
              key={link.text}
              className={className}
              id={linkedVisualization}
              onClick={link.kind == "drive_file" ? saveToLocalStorage:resetLocalStorage}
            >
              {link.icon && <i className={PorscheAftersales.className + " text-5xl"}>{link.icon}</i>}
              {link.kind == "folder" && <Folder className={clsx("size-6")}/>}
              {link.kind == "open_folder" && <FolderOpen className={clsx("size-6")}/>}
              {link.kind == "drive_file" && !link.selected && <InsertDriveFile className={clsx("size-6 {style}")}/>}
              {link.kind == "drive_file" && link.selected &&
                  <InsertDriveFileOutlined className={clsx("size-6 {style}")}/>}
              <p className={clsx("block my-auto")}>{link.text}</p>
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

  return <div className={clsx("w-full h-full flex flex-col divide-y bg-zinc-100")}>
    {HeaderComponent}
    <NavLinks navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
  </div>
}
