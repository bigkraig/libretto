import React, {JSX, useContext, useMemo} from "react";
import {Tool} from "@/lib/api/types";
import clsx from "clsx";
import {PorscheSortIcon, PSArrowDown, PSArrowRight, PSArrowUp} from "@/lib/icons";
import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getExpandedRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  SortingState,
  useReactTable
} from '@tanstack/react-table';
import {TablePagination, ThemeProvider} from "@mui/material";
import {GetToolImageUrl} from "@/lib/api";
import Image from "next/image";
import {TablePaginationActions} from "@/app/ui/Table";
import {DocumentContext, muiTheme} from "@/lib";
import {usePathname} from "next/navigation";

function InnerToolsTable({year, vehicle, items, translations}: { year: number, vehicle: string, items: Tool[], translations: Map<string, string> }) {
  const [sorting, setSorting] = React.useState<SortingState>([])

  for (let i = 0; i < items.length; i++) {
    items[i].kind = translations.get(`lbl_${items[i].kind}`.toUpperCase()) ?? items[i].kind
  }

  const columnHelper = createColumnHelper<Tool>()
  const columns: ColumnDef<Tool, any>[] = [
    columnHelper.accessor('label', {
      sortingFn: "text",
      cell: ({row, getValue}) => (
        <a id={row.original.id}> {getValue()} </a>
      ),
      header: props => "Tool denomination",
      meta: {class: "w-[300px]"}
    }),
    columnHelper.accessor('kind', {
      cell: info => info.getValue(),
      header: props => "Type",
      meta: {class: "grow"},
    }),
    columnHelper.accessor('toolDisplayNumber', {
      cell: info => <a href={`/tools/${year}/${vehicle}/${btoa(info.getValue())}`} className={clsx("inline-flex place-items-center")}>
        <PSArrowRight contentType={"_contentLink"}
                      color={"text-porschered"}/>
        <span className={clsx("hover:text-porschered transition-colors duration-300 ease-in-out")}>{info.getValue()}</span>
      </a>,
      header: props => "Tool number",
      meta: {class: "w-[270px]"},
    }),
    columnHelper.accessor('toolNumber', {
      enableSorting: false,
      cell: ({row, getValue}) => (
        <div _content-tools-table="">
          {row.getCanExpand() ? (
            <button _content-tools-table=""
                    className={clsx("rounded-full border flex items-center justify-center w-8 h-8",
                      row.getIsExpanded() ? "bg-porscheblue border-porscheblue text-white" : "")}
                    {...{
                      onClick: row.getToggleExpandedHandler(),
                      style: {cursor: 'pointer'},
                    }}
            >
              {row.getIsExpanded() ? <PSArrowUp contentType={"_contentLink"} size="text-2xl"/> :
                <PSArrowDown contentType={"_contentLink"} size="text-2xl"/>}
            </button>
          ) : ('')}

        </div>
      ),
      meta: {class: "w-[32px]"},
      header: props => "",
    }),
  ]

  const table = useReactTable({
    data: useMemo(() => items, [items]),
    columns: useMemo(() => columns, []),
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getExpandedRowModel: getExpandedRowModel(),
    onSortingChange: setSorting,
    state: {
      sorting,
    },
    getSubRows: (originalRow: Tool, index: number) => {
      if (originalRow.id == "image") {
        return undefined
      }
      return [{
        id: "image",
        label: originalRow.label,
        kind: originalRow.kind,
        toolDisplayNumber: originalRow.toolDisplayNumber,
        toolNumber: originalRow.toolNumber,
      }]
    },
    initialState: {
      pagination: {
        pageIndex: 0,
        pageSize: 10,
      },
    },
  })

  // TODO Make tool link to tool page
  return (
    <div _content-tools-table="" className="p-4 px-4 grow flex flex-col">
      <table _content-tools-table="" className="flex flex-col">
        <thead _content-tools-table="">
        {table.getHeaderGroups().map(headerGroup => (
          <tr _content-tools-table="" key={headerGroup.id} className="border-b-2 flex flex-row grow">
            {headerGroup.headers.map(header => (
              <th _content-tools-table="" className={clsx("text-left", header.id, header.column.columnDef.meta?.class)}
                  key={header.id}>
                {header.isPlaceholder ? null : (
                  <div
                    className={
                      header.column.getCanSort()
                        ? 'cursor-pointer select-none inline-flex'
                        : ''
                    }
                    onClick={header.column.getToggleSortingHandler()}
                    title={
                      header.column.getCanSort()
                        ? header.column.getNextSortingOrder() === 'asc'
                          ? 'Sort ascending'
                          : header.column.getNextSortingOrder() === 'desc'
                            ? 'Sort descending'
                            : 'Clear sort'
                        : undefined
                    }
                  >
                    {header.column.getCanSort() && ({
                      asc: <PorscheSortIcon direction={"asc"}/>,
                      desc: <PorscheSortIcon direction={"desc"}/>,
                    }[header.column.getIsSorted() as string] ?? <PorscheSortIcon direction={undefined}/>)}
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
        <tbody _content-tools-table="">
        {table.getRowModel().rows.map(row => (
          <tr _content-tools-table="" key={row.id} className="border-b flex flex-row py-1">
            {
              row.depth == 0 && row.getVisibleCells().map(cell => (
                <td _content-tools-table="" key={cell.id}
                    className={clsx(cell.column.columnDef.meta?.class, "inline-flex place-items-center py-2")}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))
            }
            {
              row.depth == 1 && <td _content-tools-table="" key={row.id + "img"} colSpan={100}>
                    <div _content-tools-table="" className="flex flex-col">
                        <p _content-tools-table="" className="font-bold">Image</p>
                        <p _content-tools-table="" className="w-[150px] p-2 border">
                            <Image _content-tools-table="" src={GetToolImageUrl(row.getValue('toolNumber'))}
                                   alt={row.getValue('toolDisplayNumber')} width={150} height={0}/>
                        </p>
                    </div>
                </td>
            }
          </tr>
        ))}
        </tbody>
      </table>
      {
        table.getFilteredRowModel().rows.length > table.getState().pagination.pageSize && <ThemeProvider theme={muiTheme}>
              <TablePagination
                  className={clsx("text-lg")}
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

export function ToolsTable({ tools, translations}: {
  tools: Tool[] | undefined,
  translations: Map<string, string>
}): JSX.Element {
  let parts = usePathname().split("?")[0].split("/");
  let year = Number(parts[2]);
  let vehicle = parts[3];
  const {isToolsTableVisible, toggleToolsTableVisible} = useContext(
    DocumentContext
  );

  if (!tools) {
    return <div _content-tools-table=""></div>
  }

  let arrow;
  if (isToolsTableVisible) {
    arrow = <PSArrowDown contentType="_content-tools-table" size="text-xl" color="text-porschered"/>
  } else {
    arrow = <PSArrowUp contentType="_content-tools-table" size="text-xl"/>
  }

  return (
    <div _content-tools-table="" className="flex flex-col flex-1 mt-4 bg-white">
      <div _content-tools-table="" className="border-b">
        <div _content-tools-table="" className="flex flex-row py-1.5 justify-between items-center mx-4 cursor-pointer" onClick={toggleToolsTableVisible}>
          <span _content-tools-table="" className="font-bold text-porschegrey">Tools</span>
          <span _content-tools-table="" className="grow h-max"/>
          <button _content-tools-table="">{arrow}</button>
        </div>
      </div>
      <div _content-tools-table="" className={clsx(isToolsTableVisible ? "block" : "hidden")} id={"tools-table"}>
        <InnerToolsTable year={year} vehicle={vehicle} items={tools} translations={translations}/>
      </div>
    </div>
  );
}