import React, { useCallback, useEffect, useImperativeHandle } from "react";

import { FieldErrors, useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import fp from "lodash/fp";
import { useMatchRefsToVals } from "../../utils/reactUtils";

import { Collapsible } from "../general/Collapsible";
import { InfoBox } from "../general/InfoBox";
import {
  advSimParamFormFields,
  dataCollectionFormFields,
  FormFieldSet,
  simParamsFormFields,
} from "../../config/formFields";
import {
  PortalRunConfig,
  portalRunConfigSchema,
  PortalRunConfigStringy,
} from "../../config/PortalRunConfig";
import { TextInputGroup } from "./TextInputGroup";
import { DataCollectionGroup } from "./DataCollectionGroup";

export type FormProps = {
  defaultValues: PortalRunConfigStringy;
  onSubmit: (
    config: PortalRunConfig,
    configStringy: PortalRunConfigStringy
  ) => void;
  onDirtinessChange: (isDirty: boolean) => void;
};

export type FormHandle = {
  submit: () => unknown;
};

export const Form = React.forwardRef(
  (props: FormProps, ref: React.Ref<FormHandle>) => {
    const {
      register,
      handleSubmit,
      getValues,
      reset,
      formState: { errors: errorsRaw, isDirty },
    } = useForm<PortalRunConfigStringy>({
      defaultValues: props.defaultValues,
      resolver: zodResolver(portalRunConfigSchema, undefined, {
        rawValues: true, // We want to get the stringy config in handleSubmit, not the parsed version
      }),
    });

    const errors = useMatchRefsToVals(errorsRaw);

    const handledOnSubmit = useCallback(
      handleSubmit((configStringy: PortalRunConfigStringy) => {
        const config = portalRunConfigSchema.parse(configStringy);
        // This is necessary for dirtiness updates to work, because the form's
        // defaultValues are cached and only get updated when reset is called
        //
        // We want the most recently submitted values to be the "defaults" for
        // dirtiness evaluation.
        reset(configStringy, {
          keepValues: true,
        });
        props.onSubmit(config, configStringy);
      }),
      [handleSubmit, getValues, props.onSubmit]
    );

    useEffect(() => {
      props.onDirtinessChange(isDirty);
    }, [isDirty]);

    useImperativeHandle(ref, () => ({
      submit: handledOnSubmit,
    }));

    return (
      <form onSubmit={handledOnSubmit}>
        {/* Hidden submit to make submit on enter work */}
        <input type="submit" className="invisible absolute" />
        <Collapsible
          title="Simulation Parameters"
          problem={sectionHasErrors(errors, simParamsFormFields)}
          defaultExpanded
        >
          <TextInputGroup
            register={register}
            errors={errors}
            fields={simParamsFormFields}
          />
        </Collapsible>
        <Collapsible
          title="Advanced Settings"
          problem={sectionHasErrors(errors, advSimParamFormFields)}
        >
          <TextInputGroup
            register={register}
            errors={errors}
            fields={advSimParamFormFields}
          />
        </Collapsible>
        <Collapsible
          title="Data Collection"
          problem={sectionHasErrors(errors, dataCollectionFormFields)}
        >
          <InfoBox>
            CSV export and desired statistics must be enabled <i>before</i>{" "}
            running simulations
          </InfoBox>
          <DataCollectionGroup register={register} errors={errors} />
        </Collapsible>
      </form>
    );
  }
);

const sectionHasErrors = (
  errors: FieldErrors<PortalRunConfigStringy>,
  fields: FormFieldSet
): boolean =>
  fields
    .map(({ configPath }) => !!fp.get(configPath)(errors))
    .some(fp.identity);
