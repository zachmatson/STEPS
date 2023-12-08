import React, {
  useState,
  useCallback,
  useRef,
  useEffect,
  MouseEventHandler,
} from "react";

import * as icons from "../icons/Icons";

export type CollapsibleProps = {
  title: string;
  defaultExpanded?: boolean;
  scrollIntoView?: boolean;
  problem?: boolean;
  settingsIcon?: React.ReactElement;
  dragHandleClassname?: string;
  children?: React.ReactNode;
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
      containerRef.current?.scrollIntoView({ block: "nearest" });
    }
  }, [isOpen]);
  // Need to stop propagation so clicks in settings icon element don't trigger open/close
  const settingsIconOnClick: MouseEventHandler<HTMLDivElement> = useCallback(
    (e) => {
      e.stopPropagation();
    },
    []
  );

  const Icon = isOpen ? icons.ChevronDown : icons.ChevronRight;

  return (
    <div ref={containerRef}>
      {/* Div for header part */}
      <div
        className={`w-full h-12 px-2 flex justify-start items-center cursor-pointer \
                    ${isOpen ? "bg-gray-300" : ""}`}
        onClick={onClick}
      >
        {props.dragHandleClassname && (
          <button className={`${props.dragHandleClassname} h-full`}>
            <icons.DragHandle className="h-5" />
          </button>
        )}
        <button
          type="button"
          className="w-full h-full py-2 flex justify-start items-center select-none text-md"
        >
          <Icon className="h-5 pr-1" />
          {props.title}
          {problem && (
            <icons.Warning className="h-5 ml-auto pr-1 text-red-600" />
          )}
        </button>
        {/* Settings Icon placed outside of HTML button element for header to prevent button nesting */}
        {props.settingsIcon && (
          <div className="h-5 pr-1">
            <div
              className="h-full cursor-[initial]"
              onClick={settingsIconOnClick}
            >
              {props.settingsIcon}
            </div>
          </div>
        )}
      </div>
      <div
        className={`px-7 pt-4 mb-1 ${isOpen ? "" : "hidden"}`}
        key="children"
      >
        {props.children}
      </div>
    </div>
  );
};
