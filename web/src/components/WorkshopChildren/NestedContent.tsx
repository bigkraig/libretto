import {ContentNested} from "@/lib/api/types";
import {ExpandMore, OpenInBrowser} from "@mui/icons-material";
import {DocumentSelection} from "@/components/DocumentSelection";
import React, {useState} from "react";

export function NestedContent({
                                obj: {
                                  inputs: {
                                    type,
                                    documentType,
                                    scenarioId,
                                    aposNumber,
                                    hkapId,
                                    activity,
                                    title
                                  }
                                }, translations
                              }: { obj: ContentNested, translations: Map<string, string> }) {
  const [open, setOpen] = useState(false);
  const [inline, setInline] = useState(false);
  const [newWindow, setNewWindow] = useState(false);

  const toggleInline = () => {
    if (newWindow) {
      setNewWindow(false);
      setInline(true);
      if (open) {
        return
      }
    }
    setInline(!inline);
    setOpen(!open);
  };

  const toggleNewWindow = () => {
    if (inline) {
      setInline(false);
      setNewWindow(true);
      if (open) {
        return
      }
    }
    setNewWindow(!newWindow);
    setOpen(!open);
  };

  return <nested_component>
    <div className="flex flex-col">
      <div _content-nested="" className="wrapper-nested-document">
        { /* Can we add a hover or whatever for "Display document here" */}
        <button _content-nested="" className="expand-button-left btn btn-primary hover:bg-porschered transition-colors duration-300 ease-in-out" type="button" onClick={toggleInline}>
          <ExpandMore className={"align-middle"} fontSize={"large"}/>
        </button>
        <div _content-nested="" className={"document-title-wrapper"}>
          {/*<a _content-nested="" id={nestedId}></a>*/}
          <button _content-nested="" className="expand-button-left btn btn-primary hover:bg-porschered transition-colors duration-300 ease-in-out" type="button"
                  onClick={toggleNewWindow}>
            <OpenInBrowser _content-nested="" className={"align-middle"} fontSize={"large"}/>
          </button>
          <span _content-nested="" className={"document-title"}>
          {aposNumber} {title}
        </span>
        </div>
      </div>
      <div _content-nested="" className="pt-4 w-[1000px]">
        {open && <DocumentSelection _content-nested="" documentType={documentType ?? ""}
                                    aposNumber={aposNumber ?? ""}
                                    inline={inline} translations={translations}/>}
      </div>
    </div>
  </nested_component>
}

