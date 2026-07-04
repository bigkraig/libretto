import React, {JSX, useContext, useMemo} from "react";
import {Part} from "@/lib/api/types";
import clsx from "clsx";
import {PorscheSortIcon, PSArrowDown, PSArrowRight, PSArrowUp} from "@/lib/icons";
import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  SortingState,
  useReactTable
} from '@tanstack/react-table';
import {TablePagination, ThemeProvider} from "@mui/material";
import {TablePaginationActions} from "@/app/ui/Table";
import {DocumentContext, muiTheme} from "@/lib";

function InnerPartsTable({items, translations}: { items: Part[], translations: Map<string, string> }) {
  const [sorting, setSorting] = React.useState<SortingState>([])
  const columnHelper = createColumnHelper<Part>()
  const columns: ColumnDef<Part, any>[] = [
    columnHelper.accessor('partNumber', {
      cell: ({row, getValue}) => (
        <span className={"inline-flex place-items-center"}><PSArrowRight contentType="_content-link" size={"text-2xl"}
                                                                         color={"text-porschered"}/><a
          id={getValue()}> {getValue()} </a></span>
      ),
      header: props => "Part number",
      meta: {class: "w-[250px]"}
    }),
    columnHelper.accessor('label', {
      cell: info => info.getValue(),
      header: props => "Description",
      meta: {class: "grow"}
    }),
    columnHelper.accessor('extendedLabel', {
      cell: info => info.getValue(),
      header: props => "Addition/Scope",
      meta: {class: "w-[350px]"}
    }),
    columnHelper.accessor('quantity', {
      cell: info => info.getValue(),
      meta: {class: "w-[120px]"},
      header: props => "Quantity",
    }),
  ]

  const table = useReactTable({
    data: useMemo(() => items, [items]),
    columns: useMemo(() => columns, []),
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onSortingChange: setSorting,
    state: {
      sorting,
    },
  })

  return (
    <div _content-parts-table="" className="p-4 px-4 grow flex flex-col">
      <table _content-parts-table="" className="flex flex-col">
        <thead _content-parts-table="">
        {table.getHeaderGroups().map(headerGroup => (
          <tr _content-parts-table="" key={headerGroup.id} className="border-b-2 flex flex-row grow">
            {headerGroup.headers.map(header => (
              <th _content-parts-table="" className={clsx("text-left", header.id, header.column.columnDef.meta?.class)}
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
        <tbody _content-parts-table="">
        {table.getRowModel().rows.map(row => (
          <tr _content-parts-table="" key={row.id} className="border-b flex flex-row py-1">
            {
              row.depth == 0 && row.getVisibleCells().map(cell => (
                <td _content-parts-table="" key={cell.id}
                    className={clsx(cell.column.columnDef.meta?.class, "inline-flex place-items-center py-2")}>
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

export function PartsTable({parts, translations}: {
  parts: Part[] | undefined,
  translations: Map<string, string>
}): JSX.Element {
  const {isPartsTableVisible, togglePartsTableVisible} = useContext(
    DocumentContext
  );

  if (!parts) {
    return <div _content-parts-table=""></div>
  }

  let arrow;
  if (isPartsTableVisible) {
    arrow = <PSArrowDown contentType="_content-parts-table" size="text-xl" color="text-porschered"/>
  } else {
    arrow = <PSArrowUp contentType="_content-parts-table" size="text-xl"/>
  }

  return (
    <div _content-parts-table="" className="flex flex-col flex-1 mt-4 bg-white">
      <div _content-parts-table="" className="border-b">
        <div _content-parts-table="" className="flex flex-row py-1.5 justify-between items-center mx-4 cursor-pointer" onClick={togglePartsTableVisible}>
          <span _content-parts-table="" className="font-bold text-porschegrey">Parts</span>
          <span _content-parts-table="" className="grow h-max"/>
          <button _content-parts-table="">{arrow}</button>
        </div>
      </div>
      <div _content-parts-table="" className={clsx(isPartsTableVisible ? "block" : "hidden")} is={"parts-table"}>
        <InnerPartsTable items={parts} translations={translations}/>
      </div>
    </div>
  );
}