import React from "react";

import { FormFieldSet } from "../../config/formFields";
import { TextInputField } from "./TextInputField";
import { FieldErrors, UseFormRegister } from "react-hook-form";
import { PortalRunConfigStringy } from "../../config/PortalRunConfig";
import { extractError } from "./extractError";

export type LabelledInputGroupProps = {
  register: UseFormRegister<PortalRunConfigStringy>;
  fields: FormFieldSet;
  errors: FieldErrors<PortalRunConfigStringy>;
};

export const TextInputGroup = React.memo(
  ({ fields, register, errors }: LabelledInputGroupProps) => (
    <>
      {fields.map(({ label, placeholder, configPath }) => {
        return (
          <TextInputField
            {...{
              label,
              placeholder,
              register,
              configPath,
              error: extractError(errors, configPath),
            }}
            key={configPath}
          />
        );
      })}
    </>
  )
);
