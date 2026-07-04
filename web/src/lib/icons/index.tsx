import clsx from "clsx";
import {PorscheAftersales} from "@/lib/fonts";
import React from "react";

export function PorscheSortIcon({direction}: { direction: string | undefined }) {
  let asc_opacity = "0.3";
  let desc_opacity = "0.3";
  if (direction == "asc" ){
    asc_opacity = "0.7";
  } else if (direction == "desc") {
    desc_opacity = "0.7";
  }
  return <svg width="14px" height="16px" className="align-middle pointer-events-none">
    <svg viewBox="0 0 14 13">
      <path d="M0 10l5 5 5-5z" fill-opacity={asc_opacity}>
      </path>
      <path d="M0 0h14v20H0z" fill="none">
      </path>
    </svg>
    <svg viewBox="0 6 14 13">
      <path d="M0 14l5-5 5 5z" fill-opacity={desc_opacity}>
      </path>
      <path d="M0 0h14v20H0z" fill="none">
      </path>
    </svg>
  </svg>
}


function PorscheIcon({ligature, props}: {
  ligature: string,
  props: Properties,
}) {
  var ct = {};
  // @ts-ignore
  ct[props.contentType] = "";

  return <i
    {...ct}
    className={clsx(PorscheAftersales.className, props.size ? props.size : "text-2xl", props.color ?? props.color)}
    onClick={props.onclick}>
    {ligature}
  </i>
}

interface Properties {
  size?: string | undefined
  color?: string | undefined
  onclick?: () => void | undefined
  contentType: string,
}

// lapo
export function PSApplication(props: Properties) {
  return PorscheIcon({ligature: "lapo", props})
}


export function PSArrowLeft(props: Properties) {
  return PorscheIcon({ligature: "arle", props})
}

export function PSArrowRight(props: Properties) {
  return PorscheIcon({ligature: "arri", props})
}

export function PSArrowUp(props: Properties) {
  return PorscheIcon({ligature: "arup", props})
}

export function PSArrowDown(props: Properties) {
  return PorscheIcon({ligature: "ardo", props})
}


/*[class^=porsche-icon-]:before, [class*=" porsche-icon-"]:before {*/
/*  font-family: Porsche-Aftersales-Icons, sans-serif !important;*/
/*  font-style: normal;*/
/*  font-weight: 400;*/
/*  speak: none;*/
/*  font-size: 24px;*/
/*  display: inline-block;*/
/*  text-decoration: inherit;*/
/*  width: 1em;*/
/*  text-align: center;*/
/*  font-variant: normal;*/
/*  text-transform: none;*/
/*  line-height: 1em;*/
/*  -webkit-font-smoothing: antialiased;*/
/*  -moz-osx-font-smoothing: grayscale*/
/*}*/
