'use client';

import Navigator from "@/app/ui/Navigator";
import WorkshopLiterature from "@/app/ui/WorkshopLiterature";
import Visualization from "@/app/ui/Visualization";
import {useSearchParams} from "next/navigation";
import React, {Suspense} from "react";
import clsx from "clsx";
import useSWR from "swr";
import {GetLinks} from "@/lib/navigator";

// useSearchParams() must sit under a Suspense boundary (Next 15+ no longer offers
// the missingSuspenseWithCSRBailout opt-out).
export default function Home() {
  return (
    <Suspense>
      <HomeContent/>
    </Suspense>
  );
}

function HomeContent() {
  const searchParams = useSearchParams()
  const vehicle = searchParams.get("vehicle");
  const year = searchParams.get("year")?Number(searchParams.get("year")):null;
  const location = searchParams.get("location")?Number(searchParams.get("location")):null;

  // Cached by (vehicle, year, location) so drilling in/out and back/forward are
  // instant instead of refetching the tree each time.
  const {data: navLinks = []} = useSWR(
    ["links", vehicle, year, location],
    () => GetLinks(vehicle, year, location),
  )

  return (
    <main className={clsx("flex flex-col gap-2 p-2 bg-paper")}>
      <div className={clsx("grid gap-2 md:h-[62vh] md:grid-cols-4")}>
        <div className={clsx("h-[46vh] md:h-full md:col-span-1")}>
          <Navigator navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
        </div>
        {/* Exploded-diagram navigator is desktop-only; on mobile the tree + document
            list are the useful surfaces. */}
        <div className={clsx("hidden md:block md:h-full md:col-span-3 border border-line bg-white")}>
          <Visualization navLinks={navLinks} location={location} vehicle={vehicle} year={year}/>
        </div>
      </div>
      <div className={clsx("min-h-[44vh]")}>
        <WorkshopLiterature location={location} vehicle={vehicle} year={year}/>
      </div>
    </main>
  );
}
