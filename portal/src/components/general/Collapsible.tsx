import React, { useState, useCallback, useRef, useEffect } from "react";

import * as icons from "../icons/Icons";

export const Collapsible = ({
  defaultExpanded = false,
  scrollIntoView = true,
  problem = false,
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
      <button
        onClick={onClick}
        type="button"
        className={`w-full h-12 p-2 flex flex-row justify-start items-center select-none cursor-pointer text-md\
                    ${isOpen ? "bg-gray-300" : ""}`}
      >
        <Icon className="h-5 pr-1" />
        {props.title}
        {problem && <icons.Warning className="h-5 pr-1 ml-auto text-red-600" />}
      </button>
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

export type CollapsibleProps = {
  title: string;
  defaultExpanded?: boolean;
  scrollIntoView?: boolean;
  problem?: boolean;
  children?: React.ReactNode;
};
