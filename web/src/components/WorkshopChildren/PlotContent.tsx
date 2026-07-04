import {ContentPlot} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";
import clsx from "clsx";
import React from "react";

export function PlotContent({obj: {inputs}, translations}: { obj: ContentPlot, translations: Map<string, string> }) {
  const plotItems = inputs;
  const children: React.JSX.Element[] = []

  plotItems.map((child) => {
    const nested = Array.isArray(child.children) && child.children.map((c, index) => {
      return <WorkshopChildren obj={c} key={index} translations={translations}/>;
    });

    children.push(<div _content-plot="" key={child.position} className={clsx("plot-element")}>
      <span _content-plot="" className={clsx("plot-information plot-index ")} id={child.id}>
        {child.position}
      </span>
      <div _content-plot="" className={clsx("plot-information")}>
        {nested && nested}
      </div>
    </div>);
  });

  return <plot_component _content-plot="">
    <div _content-plot="" className={clsx("plot-table")}>
      {children}
    </div>
  </plot_component>
}

