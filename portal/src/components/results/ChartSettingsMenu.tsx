import React, { useCallback, useState } from "react";

import * as icons from "../icons/Icons";

export type ChartSettingsMenuProps = {
  statName: string;
};

export const ChartSettingsMenu = (_: ChartSettingsMenuProps) => {
  const [expanded, setExpanded] = useState(false);
  // TODO: Clicks outside
  const onClick: React.MouseEventHandler<HTMLButtonElement> = useCallback(
    (event) => {
      setExpanded((old) => !old);
      event.stopPropagation();
    },
    []
  );

  return (
    <div className="h-full relative inline-block">
      <button className="h-full" onClick={onClick}>
        <icons.SliderSettings className="h-full" />
      </button>
      <div
        className={`${expanded ? "block" : "hidden"} absolute z-50 \
        right-0 p-3 bg-gray-50 rounded shadow-lg \
        `}
      >
        <p className="mb-1 m-0 text-left text-md">Y Scale</p>
        <div className="flex flex-nowrap mb-3.5">
          <label className="whitespace-nowrap pr-3">
            <input type="radio" />
            <span className="pl-1">Linear</span>
          </label>
          <label className="whitespace-nowrap pr-2">
            <input type="radio" />
            <span className="pl-1">Log2</span>
          </label>
        </div>
        <p className="mb-1 m-0 text-left text-md">X Scale</p>
        <div className="flex flex-nowrap">
          <label className="whitespace-nowrap pr-3">
            <input type="radio" />
            <span className="pl-1">Linear</span>
          </label>
          <label className="whitespace-nowrap pr-2">
            <input type="radio" />
            <span className="pl-1">Log2</span>
          </label>
        </div>
      </div>
    </div>
  );
};
