import React from "react";

import { FieldErrors, UseFormRegister } from "react-hook-form";

import { CheckboxField } from "./CheckboxField";
import { TextInputField } from "./TextInputField";
import { PortalRunConfigStringy } from "../../config/PortalRunConfig";
import { extractError } from "./extractError";
import {
  dataResolutionFormField,
  enableCSVFormField,
  trackedStatisticsFormFields,
} from "../../config/formFields";

export type DataCollectionGroupProps = {
  errors: FieldErrors<PortalRunConfigStringy>;
  register: UseFormRegister<PortalRunConfigStringy>;
};

export const DataCollectionGroup = (props: DataCollectionGroupProps) => (
  <>
    <CheckboxField
      label={enableCSVFormField.label}
      configPath={enableCSVFormField.configPath}
      register={props.register}
    />
    <div className="mt-0.5 pb-3.5 pl-0.5 select-none cursor-default">
      Tracked Statistics
    </div>
    {trackedStatisticsFormFields.map(({ label, configPath }) => (
      <CheckboxField
        label={label}
        configPath={configPath}
        register={props.register}
        key={configPath}
      />
    ))}
    <TextInputField
      label={dataResolutionFormField.label}
      placeholder={dataResolutionFormField.placeholder}
      configPath={dataResolutionFormField.configPath}
      register={props.register}
      error={extractError(props.errors, dataResolutionFormField.configPath)}
    />
  </>
);
