'use client';

import Navigator from "@/app/ui/Navigator";
import WorkshopLiterature from "@/app/ui/WorkshopLiterature";
import Visualization from "@/app/ui/Visualization";
import {useSearchParams} from "next/navigation";
import React, {useEffect, useState} from "react";
import clsx from "clsx";
import {GetLinks, NavigatorLink} from "@/lib/navigator";

export default function Home() {
  const searchParams = useSearchParams()
  const vehicle = searchParams.get("vehicle");
  const year = searchParams.get("year")?Number(searchParams.get("year")):null;
  const location = searchParams.get("location")?Number(searchParams.get("location")):null;

  const [navLinks, setNavLinks] = useState<NavigatorLink[]>([])

  useEffect(() => {
    GetLinks(vehicle, year, location)
      .then((data) =>
        setNavLinks(data));
  }, [vehicle, year, location])

  return (
    <main className={clsx("pt-2 pl-2 pr-2")}>
      <div className={clsx("grid h-[60vh] w-full grid-cols-4 grid-rows-4 bg-zinc-50")}>
        <div className={clsx("col-span-1 row-span-4")}>
          <Navigator navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
        </div>
        <div className={clsx("col-span-3 row-span-4 fixed-height")}>
          <Visualization navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
        </div>
      </div>
      <div className={clsx("grid w-full bg-zinc-50")}>
        <div className={clsx("")}>
          <WorkshopLiterature location={location} vehicle={vehicle} year={year}/>
        </div>
      </div>
    </main>
  );
}
