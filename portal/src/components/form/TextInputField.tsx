import React from "react";

import { FieldError, FieldPath, UseFormRegister } from "react-hook-form";
import { PortalRunConfigStringy } from "../../config/PortalRunConfig";

export type LabelledInputProps = {
  label: string;
  placeholder?: string;
  error?: FieldError;
  register: UseFormRegister<PortalRunConfigStringy>;
  configPath: FieldPath<PortalRunConfigStringy>;
};

export const TextInputField = React.memo((props: LabelledInputProps) => (
  <div className="mb-3.5">
    <div className="pb-1.5 pl-0.5 select-none cursor-default">
      {props.label}
    </div>
    <input
      type="text"
      className="w-full max-w-sm px-2.5 py-1 \
          rounded-md border-2 border-gray-300 bg-gray-50 \
          focus:placeholder:text-transparent focus:border-blue-300 focus:outline-none"
      placeholder={props.placeholder}
      {...props.register(props.configPath)}
    />
    {props.error?.message && (
      <div className="text-red-500 mt-1">{props.error.message}</div>
    )}
  </div>
));
