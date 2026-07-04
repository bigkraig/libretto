import localFont from "next/font/local";
import { IBM_Plex_Sans, IBM_Plex_Mono } from "next/font/google";

// Libretto UI type system. Plex Sans carries the interface (engineering-heritage
// grotesque); Plex Mono stamps the technical codes/indices and the wordmark.
export const PlexSans = IBM_Plex_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
  variable: "--font-sans",
  display: "swap",
});

export const PlexMono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-mono",
  display: "swap",
});

export const PorscheAftersales = localFont({
  src: [{
    path: './Porsche-Aftersales-Icons30.woff',
    weight: '400',
    style: 'normal',
  }
  //     font-family: Porsche-Aftersales-Icons, sans-serif !important;
],
  variable: '--font-porsche-aftersales'
})

export const PorscheNext = localFont({
  src: [
    {
      path: './PorscheNext4-400w.woff2',
      weight: '400',
      style: 'normal',
    },
    {
      path: './PorscheNext12-400w.woff2',
      weight: '400',
      style: 'italic',
    },
    {
      path: './PorscheNext8-700w.woff2',
      weight: '700',
      style: 'normal',
    },
    {
      path: './PorscheNext16-700w.woff2',
      weight: '700',
      style: 'italic',
    },
  ],
  variable: '--font-porsche-next'
})

export const ArialNarrow = localFont({
  src: [
    {
      path: './arialnarrow.ttf',
      weight: '400',
      style: 'normal',
    },
    {
      path: './arialnarrow_italic.ttf',
      weight: '400',
      style: 'italic',
    },
    {
      path: './arialnarrow_bold.ttf',
      weight: '700',
      style: 'normal',
    },
    {
      path: './arialnarrow_bolditalic.ttf',
      weight: '700',
      style: 'italic',
    },
  ],
  variable: '--font-arial-narrow'
})
