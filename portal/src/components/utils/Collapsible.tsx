import React, { useState, useCallback, useRef, useEffect } from "react";

import * as icons from "../icons/Icons";

export type CollapsibleProps = {
  title: string;
  defaultExpanded?: boolean;
  scrollIntoView?: boolean;
  children?: React.ReactNode;
};

export const Collapsible = ({
  defaultExpanded = false,
  scrollIntoView = true,
  ...props
}: CollapsibleProps) => {
  const containerRef = useRef<HTMLDivElement>(null);

  const [isOpen, setIsOpen] = useState(defaultExpanded);
  const onClick = useCallback(() => setIsOpen((old) => !old), []);
  useEffect(() => {
    if (isOpen && scrollIntoView) {
      containerRef.current?.scrollIntoView();
    }
  }, [isOpen]);

  const Icon = isOpen ? icons.ChevronDown : icons.ChevronRight;

  return (
    <div ref={containerRef}>
      <div
        onClick={onClick}
        className={`h-12 p-2 flex flex-row justify-start items-center select-none cursor-pointer text-md\
                    ${isOpen ? "bg-gray-300" : ""}`}
      >
        <Icon className="h-5 pr-1" />
        {props.title}
      </div>
      {/* TODO: Compare performance to conditional rendering */}
      <div
        className={`px-7 pt-4 mb-1 ${isOpen ? "" : "hidden"}`}
        key="children"
      >
        {props.children}
      </div>
    </div>
  );
};
