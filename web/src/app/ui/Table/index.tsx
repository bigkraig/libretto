import React from "react";
import clsx from "clsx";
import {TablePaginationActionsProps} from "@mui/material/TablePagination/TablePaginationActions";
import usePagination from "@mui/material/usePagination";
import {ArrowBack, ArrowForward} from "@mui/icons-material";

declare module '@tanstack/react-table' {
  interface ColumnMeta<TData, TValue> {
    class: string
  }
}

export const TablePaginationActions = (props: TablePaginationActionsProps) => {
  const {count, page, rowsPerPage, onPageChange} = props

  const {items} = usePagination({
    count: Math.ceil(count / rowsPerPage),
    onChange: (event, value) => {
      onPageChange(event as React.MouseEvent<HTMLButtonElement>, value - 1);
    },
    boundaryCount: 1,
    siblingCount: 1,
    page: page + 1,
  });

  return (<nav>
    <ul className={clsx("flex flex-row items-center place-items-center gap-x-4 ml-4")}>
      {items.map(({page, type, selected, ...item}, index) => {
        let children = null;

        switch (type) {
          case 'start-ellipsis':
            children = '…';
            break;
          case 'end-ellipsis':
            children = '…';
            break;
          case 'previous':
            children = (<button type="button" {...item}
                                className={clsx("inline-flex h-btn w-btn items-center justify-center border",
                                  item.disabled ? "bg-porschedisabledbg text-porschedisabledfg border-porschedisabledbg" : "bg-porschegrey border-porschegrey text-white hover:bg-porschered hover:border-porschered hover:text-white")}>
              <ArrowBack/></button>);
            break;
          case 'next':
            children = (<button type="button" {...item}
                                className={clsx("inline-flex h-btn w-btn items-center justify-center border",
                                  item.disabled ? "bg-porschedisabledbg text-porschedisabledfg border-porschedisabledbg" : "bg-porschegrey border-porschegrey text-white hover:bg-porschered hover:border-porschered hover:text-white")}>
              <ArrowForward/></button>);
            break;
          case 'page':
            children = (
              <button
                type="button"
                className={clsx("inline-flex h-btn w-btn items-center border border-porschelightgrey justify-center text-lg",
                  selected ? "bg-porscheblue border-porscheblue text-white hover:bg-porschered hover:border-porschered"
                    :
                    "hover:bg-porscheblue hover:border-porscheblue hover:text-white")}
                {...item}
              >
                {page}
              </button>
            );
            break;
          default:
            children = (
              <button type="button" {...item}>
                {type}
              </button>
            );
            break;
        }

        return <li key={index}>{children}</li>;
      })}
    </ul>
  </nav>);
}
