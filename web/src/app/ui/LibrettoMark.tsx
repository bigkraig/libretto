import clsx from "clsx";

// Libretto mark — a bound service booklet (the literal "libretto") with a brass
// bookmark ribbon. Booklet inherits currentColor; ribbon is the fixed brass accent.
export function LibrettoMark({className}: { className?: string }) {
  return (
    <svg viewBox="0 0 40 40" className={clsx(className)} fill="none" aria-hidden="true">
      {/* booklet body */}
      <rect x="9" y="5" width="22" height="30" rx="2.5" fill="currentColor"/>
      {/* page-stack edge (subtle depth on the fore-edge) */}
      <rect x="27.6" y="8" width="1.3" height="24" rx="0.65" fill="#000000" fillOpacity="0.18"/>
      {/* brass bookmark ribbon, swallowtail notch */}
      <path d="M21 5 V19 L23.75 16.4 L26.5 19 V5 Z" fill="#C0862C"/>
    </svg>
  );
}
