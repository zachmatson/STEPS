import React, { useState, useCallback, useRef, useEffect } from "react";

import * as icons from "../icons/Icons";

export type CollapsibleProps = {
  title: string;
  defaultExpanded?: boolean;
  scrollIntoView?: boolean;
  problem?: boolean;
  children?: React.ReactNode;
  settingsIcon?: React.ReactElement;
};

export const Collapsible = ({
  defaultExpanded = false,
  scrollIntoView = true,
  problem = false,
  ...props
}: CollapsibleProps) => {
  const containerRef = useRef<HTMLDivElement>(null);

  const [isOpen, setIsOpen] = useState(defaultExpanded);
  const [clicked, setClicked] = useState(false);
  const onClick = useCallback(() => {
    setIsOpen((old) => !old);
    setClicked(true);
  }, []);
  useEffect(() => {
    if (clicked && isOpen && scrollIntoView) {
      containerRef.current?.scrollIntoView();
    }
  }, [isOpen]);

  const Icon = isOpen ? icons.ChevronDown : icons.ChevronRight;

  return (
    <div ref={containerRef}>
      {/* TODO Make button not wrap passed in components*/}
      <button
        onClick={onClick}
        type="button"
        className={`w-full h-12 p-2 flex flex-row justify-start items-center select-none cursor-pointer text-md\
                    ${isOpen ? "bg-gray-300" : ""}`}
      >
        <Icon className="h-5 pr-1" />
        {props.title}
        <div className="h-5 ml-auto flex">
          {props.settingsIcon && (
            <div className="h-full pr-1">{props.settingsIcon}</div>
          )}
          {problem && <icons.Warning className="h-full pr-1 text-red-600" />}
        </div>
      </button>
      <div
        className={`px-7 pt-4 mb-1 ${isOpen ? "" : "hidden"}`}
        key="children"
      >
        {props.children}
      </div>
    </div>
  );
};
