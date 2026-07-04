import * as React from 'react'
import {createContext} from 'react'
import {createTheme} from "@mui/material";

export const muiTheme = createTheme({
  components: {
    MuiTablePagination: {
      styleOverrides: {
        displayedRows: `font-family: var(--font-porsche-next), sans-serif;`,
      }
    },
  }
});


export const DocumentContext = createContext({
  isToolsTableVisible: false, toggleToolsTableVisible: () => {
  },
  isTechValuesTableVisible: false, toggleTechValuesTableVisible: () => {
  },
  isPartsTableVisible: false, togglePartsTableVisible: () => {
  },
});

export function HREF(vehicle: string, year: number, node_id: number | null = null): string {
  let href = "/?vehicle=" + vehicle + "&year=" + year;
  if (node_id) {
    href += `&location=${node_id}`;
  }
  return href
}

declare global {
  namespace JSX {
    interface IntrinsicElements {
      anchor_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      image_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      link_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      list_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      mixed_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      nested_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      paragraph_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      plot_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      section_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      static_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      table_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
      warning_component: React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
    }
  }
}

export function waitVisible(elem: any, callback: any) {
  let timer: NodeJS.Timeout | null = setInterval(() => {
    if (elementVisible(elem)) {
      callback();
      if (timer) {
        clearInterval(timer);
        timer = null;
      }
    }
  }, 10);
  const tm = 5000;
  setTimeout(() => {
    if (timer) {
      clearInterval(timer);
    }
  }, tm);
}

// JQuery implementation of $elem.is(':visible')
function elementVisible(elem: any) {
  return !!(elem.offsetWidth || elem.offsetHeight || elem.getClientRects().length);
}

