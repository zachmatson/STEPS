import React, { useState, useCallback } from "react";

import * as icons from "../icons/Icons";

export type CollapsibleProps = {
  title: string;
  defaultExpanded?: boolean;
  children?: React.ReactNode;
};

export const Collapsible = ({
  defaultExpanded = false,
  ...props
}: CollapsibleProps) => {
  const [open, setOpen] = useState(defaultExpanded);
  const onClick = useCallback(() => setOpen((old) => !old), []);

  const Icon = open ? icons.ChevronDown : icons.ChevronRight;

  return (
    <div>
      <div
        onClick={onClick}
        className={`h-12 p-2 flex flex-row justify-start items-center select-none cursor-pointer text-md\
                    ${open ? "bg-gray-300" : ""}`}
      >
        <Icon className="h-5 pr-1" />
        {props.title}
      </div>
      {/* TODO: Compare performance to conditional rendering */}
      <div className={`px-7 py-4 ${open ? "" : "hidden"}`} key="children">
        {props.children}
      </div>
    </div>
  );
};
