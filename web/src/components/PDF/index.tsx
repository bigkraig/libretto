import React, {JSX, useState, useEffect, useRef, useCallback} from "react";
import {GetMediaImageUrl} from "@/lib/api";
import {Document, Page, pdfjs} from 'react-pdf';
import OpenInNewIcon from "@mui/icons-material/OpenInNew";

import 'react-pdf/dist/Page/TextLayer.css';
import 'react-pdf/dist/Page/AnnotationLayer.css';

pdfjs.GlobalWorkerOptions.workerSrc = `//unpkg.com/pdfjs-dist@${pdfjs.version}/build/pdf.worker.min.mjs`;

const getStorageKey = (id: string) => `pdf_page_${id}`;

// Nearest scrollable ancestor, or null when the window/document scrolls.
function getScrollParent(el: HTMLElement): HTMLElement | null {
  let p = el.parentElement;
  while (p) {
    const oy = getComputedStyle(p).overflowY;
    if ((oy === 'auto' || oy === 'scroll') && p.scrollHeight > p.clientHeight) return p;
    p = p.parentElement;
  }
  return null;
}

// Parse "#page=N&y=F" -> {page, frac}, or null.
function parseHash(hash: string): {page: number, frac: number} | null {
  const p = hash.match(/page=(\d+)/);
  if (!p) return null;
  const y = hash.match(/y=([\d.]+)/);
  return {page: Number(p[1]), frac: y ? Number(y[1]) : 0};
}

export function PDF({id}: {
  id: string | undefined
}): JSX.Element {
  const [numPages, setNumPages] = useState<number | null>(null);
  const [pagesRendered, setPagesRendered] = useState(0);
  const [pageWidth, setPageWidth] = useState<number>(0);
  const currentPageRef = useRef<number>(1);
  const pageRefs = useRef<Map<number, HTMLDivElement>>(new Map());
  const containerRef = useRef<HTMLDivElement>(null);
  const hasRestoredRef = useRef(false);

  // Render each page at the container's width so it fits on any screen (was a
  // fixed fraction of window.outerWidth, which broke on mobile). Capped so pages
  // don't get unreadably wide on large displays.
  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const update = () => setPageWidth(Math.min(el.clientWidth, 1100));
    update();
    const ro = new ResizeObserver(update);
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const saveCurrentPage = useCallback(() => {
    if (id && currentPageRef.current) {
      localStorage.setItem(getStorageKey(id), String(currentPageRef.current));
    }
  }, [id]);

  useEffect(() => {
    if (!id || !numPages || pagesRendered < numPages) return;

    const observer = new IntersectionObserver(
      (entries) => {
        let mostVisiblePage = currentPageRef.current;
        let maxRatio = 0;

        entries.forEach((entry) => {
          if (entry.isIntersecting && entry.intersectionRatio > maxRatio) {
            const pageNum = Number(entry.target.getAttribute('data-page'));
            if (pageNum) {
              maxRatio = entry.intersectionRatio;
              mostVisiblePage = pageNum;
            }
          }
        });

        if (maxRatio > 0) {
          currentPageRef.current = mostVisiblePage;
        }
      },
      {threshold: [0, 0.25, 0.5, 0.75, 1]}
    );

    pageRefs.current.forEach((element) => {
      observer.observe(element);
    });

    return () => observer.disconnect();
  }, [id, numPages, pagesRendered]);

  // Scroll to a page (1-based) and optionally a fraction down that page.
  const scrollToPage = useCallback((pageNum: number, frac: number) => {
    if (!(pageNum >= 1 && pageNum <= (numPages || 0))) return;
    const pageElement = pageRefs.current.get(pageNum);
    if (!pageElement) return;
    pageElement.scrollIntoView({behavior: 'instant', block: 'start'});
    if (frac > 0) {
      // nudge down into the page by the fraction-from-top encoded in the link
      const delta = frac * pageElement.offsetHeight;
      const scroller = getScrollParent(pageElement);
      if (scroller) scroller.scrollTop += delta;
      else window.scrollBy(0, delta);
    }
    currentPageRef.current = pageNum;
  }, [numPages]);

  // On load, honor a #page hash; otherwise restore the last-viewed page.
  const restoreTarget = useCallback((): {page: number, frac: number} | null => {
    const fromHash = parseHash(window.location.hash);
    if (fromHash) return fromHash;
    if (id) {
      const saved = Number(localStorage.getItem(getStorageKey(id)) || "");
      if (saved) return {page: saved, frac: 0};
    }
    return null;
  }, [id]);

  useEffect(() => {
    if (!id || !numPages || pagesRendered < numPages || hasRestoredRef.current) return;
    const t = restoreTarget();
    if (t) scrollToPage(t.page, t.frac);
    hasRestoredRef.current = true;
  }, [id, numPages, pagesRendered, restoreTarget, scrollToPage]);

  const readScroll = useCallback((): number => {
    const s = containerRef.current && getScrollParent(containerRef.current);
    return s ? s.scrollTop : window.scrollY;
  }, []);
  const writeScroll = useCallback((v: number) => {
    const s = containerRef.current && getScrollParent(containerRef.current);
    if (s) s.scrollTop = v; else window.scrollTo(0, v);
  }, []);

  // Intercept same-document link clicks in the CAPTURE phase (before pdf.js's
  // own anchor handling, which would otherwise navigate to the absolute URL and
  // leave the PDF). We stamp the CURRENT scroll position onto the current history
  // entry, then push the target as a NEW entry and scroll — so the URL stays
  // shareable AND Back returns to exactly where you jumped from.
  // Cross-document links fall through to normal navigation.
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    const onClickCapture = (e: MouseEvent) => {
      const anchor = (e.target as HTMLElement).closest('a');
      if (!anchor) return;
      let url: URL;
      try { url = new URL(anchor.getAttribute('href') || '', window.location.href); } catch { return; }
      const t = parseHash(url.hash);
      if (!t || url.pathname !== window.location.pathname) return;  // not a same-doc deep link
      e.preventDefault();
      e.stopPropagation();
      window.history.replaceState({pdfScroll: readScroll()}, '');
      window.history.pushState({pdfPage: t.page, pdfFrac: t.frac}, '', url.hash);
      scrollToPage(t.page, t.frac);
    };
    container.addEventListener('click', onClickCapture, true);
    return () => container.removeEventListener('click', onClickCapture, true);
  }, [scrollToPage, readScroll]);

  // Back/Forward: restore the saved scroll position, or scroll to the target page.
  useEffect(() => {
    const onPop = (e: PopStateEvent) => {
      const st = e.state as {pdfScroll?: number, pdfPage?: number, pdfFrac?: number} | null;
      if (st && typeof st.pdfScroll === 'number') {
        writeScroll(st.pdfScroll);
      } else if (st && st.pdfPage) {
        scrollToPage(st.pdfPage, st.pdfFrac || 0);
      } else {
        const t = parseHash(window.location.hash);
        if (t) scrollToPage(t.page, t.frac);
      }
    };
    window.addEventListener('popstate', onPop);
    return () => window.removeEventListener('popstate', onPop);
  }, [scrollToPage, writeScroll]);

  useEffect(() => {
    return () => {
      saveCurrentPage();
    };
  }, [saveCurrentPage]);

  useEffect(() => {
    window.addEventListener('beforeunload', saveCurrentPage);
    return () => window.removeEventListener('beforeunload', saveCurrentPage);
  }, [saveCurrentPage]);

  function onDocumentLoadSuccess({numPages}: { numPages: number }) {
    setNumPages(numPages);
    setPagesRendered(0);
    hasRestoredRef.current = false;
  }

  const setPageRef = (pageNum: number) => (el: HTMLDivElement | null) => {
    if (el) {
      pageRefs.current.set(pageNum, el);
    }
  };

  const onPageRenderSuccess = () => {
    setPagesRendered((prev) => prev + 1);
  };

  if (!id) {
    return <div>no media</div>;
  }

  const url = GetMediaImageUrl(id);
  return <div ref={containerRef}>
    <a href={url} target="_blank" rel="noreferrer"
       className={"mb-3 inline-flex items-center gap-1 text-sm font-medium text-brass-dim hover:text-brass"}>
      Open PDF
      <OpenInNewIcon fontSize="small"/>
    </a>
    <Document className={"mt-1"} file={url} onLoadSuccess={onDocumentLoadSuccess}>
      {pageWidth > 0 && Array.from(
        new Array(numPages),
        (el, index) => (
          <div
            key={`page_${index + 1}`}
            ref={setPageRef(index + 1)}
            data-page={index + 1}
            className="mb-4 border border-line shadow-sm"
          >
            <Page
              width={pageWidth}
              pageNumber={index + 1}
              onRenderSuccess={onPageRenderSuccess}
            />
          </div>
        ),
      )}
    </Document>
  </div>;
}