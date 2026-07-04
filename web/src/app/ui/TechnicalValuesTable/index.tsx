import React, {JSX, useContext, useMemo} from "react";
import {Techvalue} from "@/lib/api/types";
import clsx from "clsx";
import {PorscheSortIcon, PSArrowDown, PSArrowUp} from "@/lib/icons";
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
import {TablePaginationActions} from "@/app/ui/Table";
import {WorkshopChildren} from "@/components/WorkshopChildren";
import {DocumentContext, muiTheme} from "@/lib";

function InnerTechnicalValuesTable({items, translations}: { items: Techvalue[], translations: Map<string, string> }) {
  const [sorting, setSorting] = React.useState<SortingState>([])
  const columnHelper = createColumnHelper<Techvalue>()
  const columns: ColumnDef<Techvalue, any>[] = [
    columnHelper.accessor('usageLocation', {
      cell: ({row, getValue}) => (
        <a id={row.original.id}> {getValue()} </a>
      ),
      header: props => "Location",
      meta: {class: "w-[300px]"}
    }),
    columnHelper.accessor('description', {
      cell: info => info.getValue(),
      header: props => "Description",
      meta: {class: "grow"}
    }),
    columnHelper.accessor('kind', {
      cell: info => info.getValue(),
      header: props => "Type",
      meta: {class: "w-[150px]"}
    }),
    columnHelper.accessor('baseValue', {
      cell: info => info.getValue(),
      header: props => "Basic value",
      meta: {class: "w-[120px]"}
    }),
    columnHelper.accessor('tolerance1', {
      cell: info => info.getValue(),
      header: props => "Lower tolerance",
      meta: {class: "w-[120px]"}
    }),
    columnHelper.accessor('tolerance2', {
      cell: info => info.getValue(),
      header: props => "Upper tolerance",
      meta: {class: "w-[120px]"}
    }),
    columnHelper.accessor('images', {
      cell: ({row, getValue}) => (
        <div>
          {row.getCanExpand() && row.getValue("images") ? (
            <button className={clsx("rounded-full border flex items-center justify-center w-8 h-8",
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
      enableSorting: false,
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
    getSubRows: (originalRow: Techvalue, index: number) => {
      if (originalRow.id == "image") {
        return undefined
      }
      return [{
        id: "image",
        label: originalRow.usageLocation,
        description: originalRow.description,
        kind: originalRow.kind,
        baseValue: originalRow.baseValue,
        tolerance1: originalRow.tolerance1,
        tolerance2: originalRow.tolerance2,
        images: originalRow.images,
        html: originalRow.html,
      }]
    },
    initialState: {
      pagination: {
        pageIndex: 0,
        pageSize: 10,
      },
    },
  })

  return (
    <div className="pt-4 px-4 grow flex flex-col">
      <table className="flex flex-col">
        <thead>
        {table.getHeaderGroups().map(headerGroup => (
          <tr key={headerGroup.id} className="border-b-2 flex flex-row grow">
            {headerGroup.headers.map(header => (
              <th className={clsx("text-left", header.id, header.column.columnDef.meta?.class,
              )} key={header.id}>
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
        <tbody>
        {table.getRowModel().rows.map(row => (
          <tr key={row.id} className="border-b flex flex-row">
            {
              row.depth == 0 && row.getVisibleCells().map(cell => (
                <td key={cell.id}
                    className={clsx(cell.column.columnDef.meta?.class, "inline-flex place-items-center py-2")}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))
            }
            {
              row.depth == 1 && <td key={row.id + "img"} colSpan={100}>
                    <WorkshopChildren obj={row.getValue('images')} translations={translations}/>
                </td>
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
              />
          </ThemeProvider>
      }
    </div>
  )
}


export function TechnicalValuesTable({values, translations}: {
  values: Techvalue[] | undefined,
  translations: Map<string, string>
}): JSX.Element {
  const {isTechValuesTableVisible, toggleTechValuesTableVisible} = useContext(
    DocumentContext
  );

  if (!values) {
    return <div></div>
  }

  let arrow;
  if (isTechValuesTableVisible) {
    arrow = <PSArrowDown contentType={"_contentLink"} size="text-xl" color="text-porschered"/>
  } else {
    arrow = <PSArrowUp contentType={"_contentLink"} size="text-xl"/>
  }

  return (
    <div _content-technical-values-table="" className="flex flex-col flex-1 mt-4 bg-white">
      <div _content-technical-values-table="" className="border-b">
        <div _content-technical-values-table=""
             className="flex flex-row py-1.5 justify-between items-center mx-4 cursor-pointer"
             onClick={toggleTechValuesTableVisible}>
          <span _content-technical-values-table="" className="font-bold text-porschegrey">Technical values</span>
          <span _content-technical-values-table="" className="grow h-max"/>
          <button _content-technical-values-table="">{arrow}</button>
        </div>
      </div>
      <div _content-technical-values-table="" className={clsx(isTechValuesTableVisible ? "block" : "hidden")} id={"technical-values-table"}>
        <InnerTechnicalValuesTable items={values} translations={translations}/>
      </div>
    </div>
  );
}