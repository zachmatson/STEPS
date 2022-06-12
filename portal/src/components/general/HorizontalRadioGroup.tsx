import React from "react";

import { genericReactMemo } from "../../utils/reactUtils";
import { LabelledRadioButton } from "./LabelledRadioButton";

export type RadioGroupItem<Value> = {
  label: string;
  value: Value;
};

export type HorizontalRadioGroupProps<Value> = {
  title?: string;
  items: RadioGroupItem<Value>[];
  selected?: Value;
  onSelect: (value: Value) => void;
};

export const HorizontalRadioGroup = genericReactMemo(
  <Values,>({
    title,
    items,
    selected,
    onSelect,
  }: HorizontalRadioGroupProps<Values>) => (
    <>
      {title && (
        <p className="mb-1 m-0 text-left text-md select-none cursor-default">
          {title}
        </p>
      )}
      <div className="flex flex-nowrap pb-3.5 last:pb-0">
        {items.map(({ label, value }, idx) => (
          <LabelledRadioButton
            label={label}
            key={label + idx}
            checked={!!selected && selected == value}
            onCheck={() => onSelect?.(value)}
          />
        ))}
      </div>
    </>
  )
);
