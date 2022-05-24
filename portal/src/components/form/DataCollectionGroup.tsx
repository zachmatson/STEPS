import React from "react";

import { FieldErrors, UseFormRegister } from "react-hook-form";

import { CheckboxField } from "./CheckboxField";
import { TextInputField } from "./TextInputField";
import { PortalRunConfigStringy } from "../../config/config";
import { extractError } from "./extractError";
import { enableCSVFormField, trackedStatisticsFormFields } from "./formFields";

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
      />
    ))}
    <TextInputField
      label="Data Resolution"
      placeholder="Automatic Resolution"
      register={props.register}
      configPath="dataConfig.dataResolution"
      error={extractError(props.errors, "dataConfig.dataResolution")}
    />
  </>
);
