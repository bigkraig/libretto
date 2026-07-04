import {ContentTable} from "@/lib/api/types";
import {WorkshopChildren} from "@/components/WorkshopChildren/WorkshopChildren";
import React from "react";


// <ng-component>
//   <table _ngcontent-ng-c991585699="" class="table table-responsive">
//     <thead _ngcontent-ng-c991585699="">
//     <tr _ngcontent-ng-c991585699="">
//       <th _ngcontent-ng-c991585699="">
//         <ng-component _nghost-ng-c3093939457="">
//           <span _ngcontent-ng-c3093939457="">
//            Labor operation number
//           </span>
//         </ng-component>
//       </th>
//       <th _ngcontent-ng-c991585699="">
//         <ng-component _nghost-ng-c3093939457="">
//           <span _ngcontent-ng-c3093939457="">
//            Description
//           </span>
//         </ng-component>
//       </th>
//       <th _ngcontent-ng-c991585699="">
//         <ng-component _nghost-ng-c3093939457="">
//           <span _ngcontent-ng-c3093939457="">
//            I-No.
//           </span>
//         </ng-component>
//       </th>
//     </tr>
//     </thead>
//     <tbody _ngcontent-ng-c991585699="">
//     <tr _ngcontent-ng-c991585699="">
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           40155600
//          </span>
//       </td>
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           Replacing upper trailing arm (with steel chassis)
//          </span>
//       </td>
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           -
//          </span>
//       </td>
//     </tr>
//     <tr _ngcontent-ng-c991585699="">
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           40155602
//          </span>
//       </td>
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           Replacing upper trailing arm (with air suspension without all-wheel drive)
//          </span>
//       </td>
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           -
//          </span>
//       </td>
//     </tr>
//     <tr _ngcontent-ng-c991585699="">
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           40155603
//          </span>
//       </td>
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           Replacing upper trailing arm (with air suspension with all-wheel drive)
//          </span>
//       </td>
//       <td _ngcontent-ng-c991585699="">
//          <span _ngcontent-ng-c991585699="">
//           -
//          </span>
//       </td>
//     </tr>
//     </tbody>
//   </table>
// </ng-component>

//  This renders with just head/body but no data.
// should be a hidden table that can be dropped down, i have captured in quicktime video

// http://localhost:3030/v1/workshop_literature/81662848
// {
//   "type": "link",
//   "inputs": {
//   "html": "<a class=\"qv qv-techvalue\" href=\"\" target=\"PIWIS1bd04699-97a7-47bf-b905-909f2feeab98\">Tightening torque 8 Nm (5.9 ftlb.) </a>",
//     "children": [
//     {
//       "type": "table",
//                             "inputs": {
//                               "id": "PIWIS1bd04699-97a7-47bf-b905-909f2feeab98",
//                               "usageLocation": "Screws securing luggage compartment liner to body",
//                               "description": "Item 1",
//                               "kind": "Tightening torque",
//                               "baseValue": "8 Nm (5.9 ftlb.)",
//                               "tolerance1": [],
//                               "tolerance2": [],
//                               "images": [
//                                 {
//                                   "type": "mixed",
//                                   "inputs": {
//                                     "children": [
//                                       {
//                                         "type": "anchor",
//                                         "inputs": {
//                                           "id": "PIWIS94d2be91-f612-47d0-9a81-f00103ef0638"
//                                         }
//                                       },
//                                       {
//                                         "type": "image",
//                                         "inputs": {
//                                           "id": "PIWIS897637d1-874f-4de1-84f2-626e91bca3ea2",
//                                           "mediacloudSmall": "",
//                                           "mediacloudNormal": "",
//                                           "mediacloudLarge": "",
//                                           "inTable": true,
//                                           "format": "TODO (UNKNOWN)",
//                                           "key": "82197764",
//                                           "title": "Pan"
//                                         }
//                                       }
//                                     ]
//                                   }
//                                 }

export function TableContent({
                               obj: {
                                 inputs: {
                                   id,
                                   usageLocation,
                                   description,
                                   kind,
                                   baseValue,
                                   tolerance1,
                                   tolerance2,
                                   pgwide,
                                   data,
                                   header,
                                   title,
                                   images
                                 }
                               }, translations
                             }: {
  obj: ContentTable,
  translations: Map<string, string>
}) {
//     id?: string;
//     usageLocation?: string;
//     description?: string;
//     kind?: string;
//     baseValue?: string;
//     tolerance1?: string[];
//     tolerance2?: string[];
//     pgwide?: boolean;
//     data?: Children;
//     header?: Children;
//     title?: string;
//     images?: Children;

  let headerRows: React.JSX.Element[] = [];
  if (Array.isArray(header)) {
    header.map((headerRow, index) => {
      let row: React.JSX.Element[] = [];
      if (Array.isArray(headerRow)) {
        headerRow.map((headerCell, index) => {
          row.push(<th _content-table="" key={index}><WorkshopChildren translations={translations} obj={headerCell}/>
          </th>);
        });
      }
      headerRows.push(<tr _content-table="" key={index}>{row}</tr>);
    });
  }

  let dataRows: React.JSX.Element[] = [];
  if (Array.isArray(data)) {
    data.map((dataRow, index) => {
      let row: React.JSX.Element[] = [];
      if (Array.isArray(dataRow)) {
        dataRow.map((dataCell, index) => {
          if (Array.isArray(dataCell)) {
            row.push(<td _content-table="" key={index} colSpan={dataCell[1] as number} rowSpan={dataCell[2] as number}>
              <WorkshopChildren obj={dataCell[0]} translations={translations}/>
            </td>);
          } else {
            row.push(<td _content-table="" key={index}><WorkshopChildren translations={translations} obj={dataCell}/>
            </td>);
          }
        });
      }
      dataRows.push(<tr _content-table="" key={index}>{row}</tr>)
    });
  }

  return <div _content-table="">
    {title && <h3 _content-table="">{title}</h3>}
    <table _content-table="" className={"table table-responsive"}>
      <thead _content-table="">
      {headerRows && headerRows}
      </thead>
      <tbody _content-table="">
      {dataRows && dataRows}
      </tbody>
    </table>
  </div>
}
