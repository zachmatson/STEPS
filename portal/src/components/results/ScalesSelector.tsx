import React, { useCallback } from "react";
import { AxisScale, ChartScales, XAxisUnits } from "../../config/chartConfig";
import {
  HorizontalRadioGroup,
  RadioGroupItem,
} from "../general/HorizontalRadioGroup";

const scaleRadioItems: RadioGroupItem<AxisScale>[] = [
  {
    label: "Linear",
    value: "linear",
  },
  {
    label: "Log2",
    value: "log2",
  },
];

const xAxisUnitRadioItems: RadioGroupItem<XAxisUnits>[] = [
  {
    label: "Generations",
    value: "generations",
  },
  {
    label: "Transfers",
    value: "transfers",
  },
];

export type ScalesSelectorProps = {
  value: ChartScales;
  onChange?: (value: ChartScales) => void;
};

export const ScalesSelector = ({ value, onChange }: ScalesSelectorProps) => {
  const [onSelectYScale, onSelectXScale, onSelectXUnits] = (
    ["yScale", "xScale", "xUnits"] as (keyof ChartScales)[]
  ).map((field) =>
    useCallback(
      (newValue: string) => {
        if (newValue != value[field]) {
          onChange?.({ ...value, [field]: newValue });
        }
      },
      [value, onChange]
    )
  );

  return (
    <div className="p-3 bg-gray-50 rounded shadow-lg">
      <HorizontalRadioGroup
        title={"Y Scale"}
        items={scaleRadioItems}
        selected={value.yScale}
        onSelect={onSelectYScale}
      />
      <HorizontalRadioGroup
        title={"X Scale"}
        items={scaleRadioItems}
        selected={value.xScale}
        onSelect={onSelectXScale}
      />
      <HorizontalRadioGroup
        title={"X Units"}
        items={xAxisUnitRadioItems}
        selected={value.xUnits}
        onSelect={onSelectXUnits}
      />
    </div>
  );
};
