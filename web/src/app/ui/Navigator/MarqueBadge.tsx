"use client";

import {useState} from "react";
import clsx from "clsx";

// Monogram stand-in until an official emblem SVG is dropped in public/marques/.
const MARQUE_LABEL: { [key: string]: string } = {
  porsche: "P",
  ferrari: "F",
  audi: "A",
  lamborghini: "L",
  mclaren: "M",
  astonmartin: "AM",
  bentley: "B",
  maserati: "MA",
};

export function MarqueBadge({marque}: { marque?: string }) {
  const slug = (marque || "").toLowerCase();
  const [failed, setFailed] = useState(false);
  const showEmblem = slug.length > 0 && !failed;

  // Emblems render bare (each marque owns its silhouette — shield, rings, scudetto);
  // a fixed-width slot keeps the vehicle names aligned. The circular chip is only
  // the monogram stand-in.
  if (showEmblem) {
    // Light plate so thin/dark emblems (Audi rings, Porsche crest) stay legible on
    // the dark sidebar; colored ones (Ferrari) sit on it fine too.
    return (
      <span className={clsx("inline-flex h-7 w-9 shrink-0 items-center justify-center rounded-sm bg-white")}>
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={`/marques/${slug}.svg`}
          alt={slug}
          className={clsx("max-h-5 max-w-7 object-contain")}
          onError={() => setFailed(true)}
        />
      </span>
    );
  }

  return (
    <span className={clsx(
      "inline-flex size-7 shrink-0 items-center justify-center overflow-hidden",
      "rounded-full border border-line bg-white",
    )}>
      <span className={clsx("font-mono text-[11px] font-semibold tracking-tight text-ink")}>
        {MARQUE_LABEL[slug] ?? "•"}
      </span>
    </span>
  );
}
