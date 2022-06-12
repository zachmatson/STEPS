import React, { useCallback } from "react";
import { AxisScale, ChartScales } from "../../config/chartConfig";
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

export type ScalesSelectorProps = {
  value: ChartScales;
  onChange?: (value: ChartScales) => void;
};

export const ScalesSelector = ({ value, onChange }: ScalesSelectorProps) => {
  const [onSelectY, onSelectX] = (["y", "x"] as (keyof ChartScales)[]).map(
    (field) =>
      useCallback(
        (newValue: AxisScale) => {
          onChange?.({ ...value, [field]: newValue });
        },
        [value, onChange]
      )
  );

  return (
    <div className="p-3 bg-gray-50 rounded shadow-lg">
      <HorizontalRadioGroup
        title={"Y Scale"}
        items={scaleRadioItems}
        selected={value.y}
        onSelect={onSelectY}
      />
      <HorizontalRadioGroup
        title={"X Scale"}
        items={scaleRadioItems}
        selected={value.x}
        onSelect={onSelectX}
      />
    </div>
  );
};
