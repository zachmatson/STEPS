import React, { useCallback, useEffect, useImperativeHandle } from "react";

import {
  FieldErrors,
  useForm,
  UseFormGetValues,
  UseFormHandleSubmit,
  UseFormReset,
} from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import fp from "lodash/fp";

import { Collapsible } from "../general/Collapsible";
import { InfoBox } from "../general/InfoBox";
import {
  advSimParamFormFields,
  dataCollectionFormFields,
  FormFieldSet,
  simParamsFormFields,
} from "./formFields";
import {
  portalRunConfigSchema,
  PortalRunConfigStringy,
} from "../../config/config";
import { TextInputGroup } from "./TextInputGroup";
import { useMatchRefsToVals } from "../../utils/useMatchRefsToVals";
import { DataCollectionGroup } from "./DataCollectionGroup";

export type FormProps = {
  defaultValues: PortalRunConfigStringy;
  onSubmit: (data: PortalRunConfigStringy) => void;
  onDirtinessChange: (isDirty: boolean) => void;
};

export type FormHandle = {
  getValues: UseFormGetValues<PortalRunConfigStringy>;
  reset: UseFormReset<PortalRunConfigStringy>;
  submit: ReturnType<UseFormHandleSubmit<PortalRunConfigStringy>>;
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
      resolver: zodResolver(portalRunConfigSchema),
    });
    const errors = useMatchRefsToVals(errorsRaw);

    const handledOnSubmit = useCallback(handleSubmit(props.onSubmit), [
      handleSubmit,
      props.onSubmit,
    ]);

    useEffect(() => {
      props.onDirtinessChange(isDirty);
    }, [isDirty]);

    useImperativeHandle(ref, () => ({
      getValues,
      reset,
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
          title="Advanced Simulation Parameters"
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
