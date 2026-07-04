/** @type {import('tailwindcss').Config} */

import type {Config} from "tailwindcss";

const config: Config = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],

  theme: {
    fontSize: {
      sm: '0.8rem',
      base: '1rem',
      lg: '16px',
      xl: '18px',
      '2xl': '20px',
      '3xl': '1.953rem',
      '4xl': '2.441rem',
      '5xl': '3.052rem',
    },
    extend: {
      colors: {
        // Libretto identity — marque-neutral, instrument-grade.
        'ink': '#16181D',        // sidebar / headers
        'ink-2': '#22252C',      // raised ink surface
        'paper': '#FAFAFA',      // work area
        'line': '#E6E6E4',       // hairline dividers
        'muted': '#6B6E73',      // secondary text
        'brass': '#C0862C',      // single heritage accent
        'brass-dim': '#9A6D24',
        'brass-wash': '#F6EEDD', // faint accent fill (active row)
        // Porsche palette retained for the document viewer / legacy pieces.
        'cat1': 'rgb(255, 210, 50)',
        'catinfo': 'rgb(175, 177, 178)',
        'porschebase': 'rgb(33, 37, 41)',
        'porschelightgrey': 'rgb(123, 128, 131)',
        'porschegrey': '#494e51', // 'rgb(73, 78, 81)',
        'porscheblue': 'rgb(21, 87, 126)',
        'porschered': '#d5001c',
        'porschedisabledbg': 'rgb(175, 177, 178)',
        'porschedisabledfg': 'rgb(227, 228, 228)',
        'porschelightblue': '#3d60ed',
      },
      spacing: {
        'btn': '34px',
      },
      fontFamily: {
        sans: ['var(--font-sans)', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['var(--font-mono)', 'ui-monospace', 'monospace'],
        porsche: ['var(--font-porsche-next)']
      }
    }
  },

  plugins: [],
};
export default config;
