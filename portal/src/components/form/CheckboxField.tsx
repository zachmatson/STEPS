import { FieldPath, UseFormRegister } from "react-hook-form";
import { PortalRunConfigStringy } from "../../config/PortalRunConfig";
import React from "react";

export type CheckboxFieldProps = {
  label: string;
  register: UseFormRegister<PortalRunConfigStringy>;
  configPath: FieldPath<PortalRunConfigStringy>;
};

export const CheckboxField = React.memo((props: CheckboxFieldProps) => (
  <div className="pb-3.5 pl-0.5">
    <label className="cursor-pointer flex items-center">
      <input
        type="checkbox"
        className={
          "appearance-none w-5 h-5 border-gray-300 border-2 rounded mr-2 cursor-pointer bg-gray-50 \
           flex justify-center items-center \
           checked:before:h-3 checked:before:w-3 checked:before:rounded-sm checked:before:bg-blue-600"
        }
        {...props.register(props.configPath)}
      />
      <span className="select-none">{props.label}</span>
    </label>
  </div>
));
