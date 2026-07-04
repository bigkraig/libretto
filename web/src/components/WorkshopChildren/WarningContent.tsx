import {ContentWarning} from "@/lib/api/types";
import clsx from "clsx";
import WarningIcon from '@mui/icons-material/Warning';
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";
import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import {ArrowForward, InfoOutlined} from "@mui/icons-material";
import React from "react";

export function WarningContent({
                                 obj: {
                                   inputs: {
                                     category,
                                     children,
                                     consequences,
                                     actions,
                                     source
                                   }
                                 }, translations
                               }: {
  obj: ContentWarning,
  translations: Map<string, string>
}) {
  let header;
  if (category == "0Hinweis") { // Notice
    header = <div _content-warning="" className="cat0 warning-heading">
      <div _content-warning="" className="background-cat0 warning-sign">
        <InfoOutlined _content-warning="" className={clsx("text-4xl")}/>
        <span _content-warning="">NOTIFICATION</span>
      </div>
    </div>
  } else if (category == "1Vorsicht") { // Caution
    header = <div _content-warning="" className="cat1 warning-heading">
      <div _content-warning="" className="background-cat1 warning-sign">
        <WarningIcon _content-warning="" className={clsx("text-4xl")}/>
        <span _content-warning="">CAUTION</span>
      </div>
    </div>
  } else if (category == "2Warnung") { // Warning
    header = <div _content-warning="" className="cat2 warning-heading">
      <div _content-warning="" className="background-cat2 warning-sign">
        <WarningIcon _content-warning="" className={clsx("text-4xl")}/>
        <span _content-warning="">WARNING</span>
      </div>
    </div>
  } else if (category == "3Gefahr") { // Danger
    header = <div _content-warning="" className="cat3 warning-heading">
      <div _content-warning="" className="background-cat3 warning-sign">
        <WarningIcon _content-warning="" className={clsx("text-4xl")}/>
        <span _content-warning="">DANGER</span>
      </div>
    </div>
  } else if (category == "Information") {
    header = <div _content-warning="" className="cat-info warning-heading">
      <div _content-warning="">
        <i _content-warning=""><InfoOutlinedIcon/></i>
        <span _content-warning="">Information</span>
      </div>
    </div>
  } else {
    console.log("uh unknown category", category)
  }

  const consequence = <ul _content-warning="" className="consequence">
    {
      Array.isArray(consequences) && consequences?.map((consequence, index) => {
        return (
          <li _content-warning="" key={index}><WorkshopChildren obj={consequence} translations={translations}/></li>)
      })
    }
  </ul>;

  const nested = Array.isArray(children) && children.map((c, index) => {
    return <WorkshopChildren obj={c} key={index} translations={translations}/>;
  });

  const action = Array.isArray(actions) && actions.map((a, index) => {
    return <div _content-warning="" key={index} className="action">
      <i _content-warning=""><ArrowForward/></i>
      <span _content-warning=""><WorkshopChildren obj={a} translations={translations}/></span>
    </div>
  });

  return <warning_component _content-warning="">
    <div _content-warning="">
      {header}
      <div _content-warning="" className={clsx(
        category == "0Hinweis" && "border-cat0 warning-content",
        category == "1Vorsicht" && "border-cat1 warning-content",
        category == "2Warnung" && "border-cat2 warning-content",
        category == "3Gefahr" && "border-cat3 warning-content",
        category == "Information" && "border-cat-info warning-content",
      )}>
        {nested && nested}
        <div _content-warning="" className="warning-source">{source}</div>
        {consequence}
        {action}
      </div>
    </div>
  </warning_component>
}

