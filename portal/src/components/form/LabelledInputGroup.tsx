import React from "react";

import fp from "lodash/fp";

import { FormFieldSet } from "./formFields";
import { LabelledInput } from "./LabelledInput";
import { FieldError, FieldErrors, UseFormRegister } from "react-hook-form";
import { PortalRunConfigStringy } from "../../config/config";

export type LabelledInputGroupProps = {
  register: UseFormRegister<PortalRunConfigStringy>;
  fields: FormFieldSet;
  errors: FieldErrors<PortalRunConfigStringy>;
};

export const LabelledInputGroup = React.memo(
  ({ fields, register, errors }: LabelledInputGroupProps) => (
    <>
      {fields.map(({ label, placeholder, configPath }) => {
        const error: FieldError | undefined = fp.get(configPath)(errors);

        return (
          <LabelledInput
            {...{
              label,
              placeholder,
              error,
              register,
              configPath,
            }}
            key={configPath}
          />
        );
      })}
    </>
  )
);
