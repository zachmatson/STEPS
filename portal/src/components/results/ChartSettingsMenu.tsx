import React, { useCallback, useEffect, useRef, useState } from "react";
import { ChartScales } from "../../config/chartConfig";

import * as icons from "../icons/Icons";
import { ScalesSelector } from "./ScalesSelector";

export type ChartSettingsMenuProps = {
  scalesValue: ChartScales;
  onScalesChange: (value: ChartScales) => void;
};

export const ChartSettingsMenu = ({
  scalesValue,
  onScalesChange,
}: ChartSettingsMenuProps) => {
  const [expanded, setExpanded] = useState(false);

  const onClickOutside: React.FocusEventHandler<HTMLDivElement> = useCallback(
    (event) => {
      setExpanded(false);
      event.stopPropagation();
    },
    []
  );
  const onIconClick: React.MouseEventHandler<HTMLButtonElement> = useCallback(
    (event) => {
      setExpanded((old) => !old);
      event.stopPropagation();
    },
    []
  );

  const popupRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (expanded) {
      popupRef.current?.focus();
    }
  }, [popupRef.current, expanded]);

  return (
    <div className="h-full relative inline-block">
      <button className="h-full cursor-pointer" onClick={onIconClick}>
        <icons.SliderSettings className="h-full" />
      </button>
      <div
        className={`\
          ${expanded ? "block" : "hidden"} \
          absolute z-50 right-0 focus:outline-none
        `}
        ref={popupRef}
        onBlur={onClickOutside}
        tabIndex={-1}
      >
        <ScalesSelector value={scalesValue} onChange={onScalesChange} />
      </div>
    </div>
  );
};
