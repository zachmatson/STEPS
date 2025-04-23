import React, { useRef } from "react";

import Popup from "reactjs-popup";

import { AvailableScales, ChartScales } from "../../config/chartConfig";
import * as icons from "../icons/Icons";
import { ScalesSelector } from "./ScalesSelector";

export type ChartSettingsMenuProps = {
  scalesValue: ChartScales;
  onScalesChange: (value: ChartScales) => void;
  availableScales: AvailableScales;
};

export const ChartSettingsMenu = ({
  scalesValue,
  onScalesChange,
  availableScales,
}: ChartSettingsMenuProps) => {
  const emptyDivRef = useRef<HTMLDivElement>(null);

  return (
    <div className="h-full relative inline-block">
      <Popup
        trigger={
          <button className="h-full cursor-pointer">
            <icons.SliderSettings className="h-full" />
          </button>
        }
        position="bottom right"
        arrow={false}
        onOpen={() => emptyDivRef.current?.focus()}
      >
        {/*
          The popup library focuses the radio buttons inside ScalesSelector for some reason when the popup is expanded,
          this empty div can be used to quietly take the focus instead
        */}
        <div className="h-0 w-0 m-0 p-0" tabIndex={-1} ref={emptyDivRef} />
        <ScalesSelector
          value={scalesValue}
          onChange={onScalesChange}
          availableScales={availableScales}
        />
      </Popup>
    </div>
  );
};
