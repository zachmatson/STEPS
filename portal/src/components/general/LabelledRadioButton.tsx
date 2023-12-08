import React, { ChangeEvent, MouseEvent, useCallback } from "react";

export type LabelledRadioButtonProps = {
  label: string;
  checked: boolean;
  onCheck: () => void;
};

export const LabelledRadioButton = React.memo(
  ({ checked, onCheck, label }: LabelledRadioButtonProps) => {
    const onClick = useCallback(
      (e: MouseEvent<unknown> | ChangeEvent<unknown>) => {
        if (!checked) {
          onCheck?.();
        }
        e.stopPropagation();
      },
      [onCheck]
    );

    return (
      <label
        className="whitespace-nowrap pr-3 last:pr-[inherit] cursor-pointer select-none inline-flex justify-start items-center"
        onClick={onClick}
      >
        <input
          type="radio"
          className={
            "appearance-none w-[1.125rem] h-[1.125rem] border-gray-300 border-2 rounded-[1.125rem] mr-1 cursor-pointer bg-gray-50 \
            flex justify-center items-center \
            checked:before:h-[0.625rem] checked:before:w-[0.625rem] checked:before:rounded-[0.625rem] checked:before:bg-blue-600"
          }
          checked={checked}
        />
        <span>{label}</span>
      </label>
    );
  }
);
