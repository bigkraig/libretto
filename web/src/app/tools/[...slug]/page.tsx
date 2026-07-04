'use client';

import {usePathname} from "next/navigation";
import React, {useEffect, useMemo, useState} from "react";
import {ArrowBack, Info, InsertDriveFile, OpenInBrowser, PictureAsPdf} from "@mui/icons-material";
import clsx from "clsx";
import {GetToolData, GetToolImageUrl, GetTranslations} from "@/lib/api";
import {IReferencingDocument, IToolData, IToolDistributor} from "@/lib/api/types";
import Image from "next/image";
import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  useReactTable
} from "@tanstack/react-table";
import {TablePagination, ThemeProvider} from "@mui/material";
import {muiTheme} from "@/lib";
import {TablePaginationActions} from "@/app/ui/Table";
import {Dialog, DialogBackdrop, DialogPanel, DialogTitle} from "@headlessui/react";
import {CloseIcon} from "next/dist/client/components/react-dev-overlay/internal/icons/CloseIcon";

function back() {
  window.history.go(-1);
  return false;
}

export default function Home() {
  let parts = usePathname().split("?")[0].split("/");
  let year = Number(parts[2]);
  let vehicle = parts[3];
  let tool_id = parts[4];

  let [toolData, setToolData] = useState<IToolData>()
  let [translations, setTranslations] = useState<Map<string, string>>()
  let [supplierDetailsModal, setSupplierDetailsModal] = useState<IToolDistributor | undefined>(undefined)

  useEffect(() => {
    if (!tool_id) {
      return
    }

    GetToolData(year, vehicle, tool_id).then(content => setToolData(content));
    GetTranslations().then(content => setTranslations(content));
  }, [tool_id])

  if (!toolData) {
    return
  }
  if (!translations) {
    return
  }

  return (
    <main>
      <div className={clsx("grid w-full grid-rows-[7rem,100%] grid-cols-1 bg-zinc-100")}>
        <div id="navBar" className={clsx("row-span-1 col-span-5 bg-white inline-flex items-center")}>
          <div className={clsx("flex pt-3 pb-3 bg-white")}>
            <div
              className={clsx("p-1.5 bg-zinc-700 text-white my-auto hover:bg-red-700 transition-colors duration-300 ease-in-out cursor-pointer")}
              onClick={back}>
              <ArrowBack className={clsx('m-auto')}/>
            </div>
            <span
              className={clsx("text-xl font-bold my-auto pl-4")}>{toolData.title} {toolData.tool_number_pag}</span>
          </div>
        </div>
        <div className={clsx("row-span-1 print:overflow-visible mt-4 mx-9 bg-white border border-porschedisabledfg")}>
          <div className={clsx("border border-porschedisabledbg mt-9 pb-4 mx-9")}>
            <span className={clsx("text-[12px] z-10 mx-[6px] px-[1px] text-start top-[-12px] relative bg-white")}>General Information</span>
            <div className={clsx("grid grid-cols-12 px-6")}>
              <div className="col-span-4">Tool type</div>
              <div className="col-span-8">{toolData.tool_type}</div>
              <div className="col-span-4">Dealer classification</div>
              <div className="col-span-8">{toolData.dealer_classification}</div>
              <div className="col-span-4">Status</div>
              <div className="col-span-8">{translations.get(`LBL_TOOL_STATUS_${toolData.state}`)}</div>
              <div className="col-span-4">Utilization/Description</div>
              <div className="col-span-8">{toolData.description}</div>
              <div className="col-span-4">Photo</div>
              <div className="col-span-8">
                <Image _content-tools-table="" src={GetToolImageUrl(toolData.title)} alt={toolData.tool_number_pag}
                       width={150} height={0}/>
              </div>
            </div>
          </div>

          <div className={clsx("border border-porschedisabledbg mt-9 pb-4 mx-9")}>
              <span className={clsx("text-[12px] z-10 mx-[6px] px-[1px] text-start top-[-12px] relative bg-white")}>
                Suppliers
              </span>

            <div className={clsx("grid w-full")}>
              <table className="mx-8 text-left mb-8">
                <thead className={"border-b-2"}>
                <tr>
                  <th className={"w-[200px]"}>
                    Part number
                  </th>
                  <th>
                    Supplier
                  </th>
                  <th className={"w-[40px]"}>
                  </th>
                  <th className="detail-toggle">
                  </th>
                </tr>
                </thead>

                <tbody>
                {
                  toolData.distributors?.map((distributor, index) => {
                    return <tr key={index} className={"border-b"}>
                      <td>
                        <span title={distributor.part_number}>{distributor.part_number}</span>
                      </td>
                      <td>
                        <span className="title-link" title={distributor.name}>{distributor.name}</span>
                      </td>
                      <td>
                        <button type="button"
                                className={clsx("inline-flex py-2 h-btn w-btn items-center justify-center",
                                  "transition-colors duration-700 ease-in-out",
                                  "border-porschegrey text-porschegrey hover:text-porschered")}
                                onClick={() => setSupplierDetailsModal(distributor)}>
                          <Info/></button>
                      </td>
                      <td className="toggle-col has-action-icons"></td>
                    </tr>
                  })
                }
                </tbody>
              </table>
            </div>

          </div>

          {
            toolData.referencing_documents && <div className={clsx("border border-porschedisabledbg mt-9 mx-9")}>
                  <span className={clsx("text-[12px] z-10 mx-[6px] px-[1px] text-start top-[-12px] relative bg-white")}>
                      Locations
                  </span>
                  <InnerTable year={year} vehicle={vehicle} items={toolData.referencing_documents}
                              translations={translations}/>
              </div>
          }
        </div>
      </div>

      {supplierDetailsModal &&
          <Dialog open={supplierDetailsModal && true} as="div" className="relative z-20 focus:outline-none"
                  onClose={() => setSupplierDetailsModal(undefined)}>
              <DialogBackdrop className="fixed inset-0 bg-black/50"/>
              <div className="fixed inset-0 z-20 w-screen overflow-y-auto">
                  <div className="flex min-h-full items-center justify-center p-4">
                      <DialogPanel transition
                                   className="flex flex-col bg-white w-[800px] backdrop-blur-2xl duration-300 ease-out data-[closed]:transform-[scale(95%)] data-[closed]:opacity-0"
                      >
                          <DialogTitle as="div"
                                       className="inline-flex flex-auto text-white font-bold bg-porschelightgrey px-[30px] pt-[25px] pb-[15px]">
                              Supplier {supplierDetailsModal.name}
                              <div className={"grow"}/>
                              <button onClick={() => setSupplierDetailsModal(undefined)}>
                                  <CloseIcon/>
                              </button>
                          </DialogTitle>

                          <div className="m-[30px] grid grid-cols-[266px,1fr]">
                              <span className={"border-b py-2"}>Code</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.distributor_code}</span>

                              <span className={"border-b py-2"}>Name</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.name}</span>


                              <span className={"border-b py-2"}>Street</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.street}</span>


                              <span className={"border-b py-2"}>Zip code</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.zip}</span>

                              <span className={"border-b py-2"}>City</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.city}</span>


                              <span className={"border-b py-2"}>Telephone number</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.phone}</span>

                              <span className={"border-b py-2"}>Fax</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.fax}</span>

                              <span className={"border-b py-2"}>E-mail</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.email}</span>

                              <span className={"border-b py-2"}>Web address</span>
                              <span className={"border-b py-2"}>{supplierDetailsModal.web}</span>
                          </div>

                          <div
                              className="inline-flex flex-auto text-white bg-porschedisabledfg place-items-center h-[100px] px-[30px] pt-[25px] pb-[15px]">
                              <div className={"grow"}/>
                              <button className="flex bg-porschegrey place-items-center pl-[8px] !h-[34px] hover:bg-red-700 transition-colors duration-300 ease-in-out"
                                      onClick={() => setSupplierDetailsModal(undefined)}>
                                  <CloseIcon/>
                                  <span className="align-middle ml-[2px] pr-[15px]">Close</span>
                              </button>
                          </div>

                      </DialogPanel>
                  </div>
              </div>
          </Dialog>
      }

    </main>
  )
}


function InnerTable({
                      year, vehicle, items, translations
                    }: {
  year: number,
  vehicle: string,
  items: IReferencingDocument[],
  translations: Map<string, string>
}) {
  const columnHelper = createColumnHelper<IReferencingDocument>()
  const columns: ColumnDef<IReferencingDocument, any>[] = [
    columnHelper.accessor('file_format',
      {
        cell: ({row, getValue}) => (
          <span className={"inline-flex place-items-center"}>
            {row.original.file_format == "xml" ? <InsertDriveFile/> : <PictureAsPdf/>}
        </span>
        ),
        header: props => "",
        meta: {class: "w-[40px]"}
      }),
    columnHelper.accessor('hkap_id', {
      cell: ({row, getValue}) => (
        <a href={`/documents/${year}/${vehicle}/${row.original.hkap_id}`}>
          {row.original.vehicle_component_with_document_index} {row.original.title}
        </a>
      ),
      header: props => "",
      meta: {class: "grow"}
    }),
    columnHelper.accessor('language_code', {
      cell: ({row, getValue}) => (
        <a href={`/documents/${year}/${vehicle}/${row.original.hkap_id}`}
           className={clsx("rounded-full border flex items-center justify-center w-8 h-8",
             "transition-colors duration-700 ease-in-out",
             "hover:text-porschered")}>
          <OpenInBrowser/>
        </a>
      ),
      header: props => "",
      meta: {class: "w-[40px]"}
    }),
    columnHelper.accessor('vehicle_component_with_document_index', {
      cell: info => {
      },
      meta: {class: "w-[35px]"},
      header: props => "",
    }),
  ]

  const table = useReactTable({
    data: useMemo(() => items, [items]),
    columns: useMemo(() => columns, []),
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: {
      pagination: {
        pageSize: 20, //custom default page size
      },
    },
  })

  if (!items) {
    return <div></div>
  }

  return (
    <div className="p-4 px-4 grow flex flex-col">
      <table className="flex flex-col">
        <thead>
        {table.getHeaderGroups().map(headerGroup => (
          <tr key={headerGroup.id} className="border-b-2 flex flex-row grow">
            {headerGroup.headers.map(header => (
              <th className={clsx("text-left", header.id, header.column.columnDef.meta?.class)}
                  key={header.id}>
                {header.isPlaceholder ? null : (
                  <div>
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext()
                    )}
                  </div>
                )}
              </th>
            ))}
          </tr>
        ))}
        </thead>
        <tbody>
        {table.getRowModel().rows.map(row => (
          <tr key={row.id} className="border-b flex flex-row">
            {
              row.getVisibleCells().map(cell => (
                <td key={cell.id}
                    className={clsx(cell.column.columnDef.meta?.class, "inline-flex place-items-center py-1")}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))
            }
          </tr>
        ))}
        </tbody>
      </table>
      {
        table.getFilteredRowModel().rows.length > table.getState().pagination.pageSize &&
          <ThemeProvider theme={muiTheme}>
              <TablePagination
                  className={clsx("text-lg mt-4")}
                  component="div"
                  classes={{displayedRows: clsx("!text-base text-porschegrey")}}
                  count={table.getFilteredRowModel().rows.length}
                  page={table.getState().pagination.pageIndex}
                  rowsPerPage={table.getState().pagination.pageSize}
                  rowsPerPageOptions={[]} // disable rows per page
                  labelDisplayedRows={({from, to, count}) => `${from} to ${to} of ${count} entries`}
                  onPageChange={(_, page) => {
                    table.setPageIndex(page)
                  }}
                  ActionsComponent={TablePaginationActions}
              /></ThemeProvider>
      }
    </div>
  )
}
