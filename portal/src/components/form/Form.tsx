import React, { useCallback, useEffect, useImperativeHandle } from "react";

import {
  useForm,
  UseFormGetValues,
  UseFormHandleSubmit,
} from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import fp from "lodash/fp";

import { Collapsible } from "../utils/Collapsible";
import { InfoBox } from "../utils/InfoBox";
import { advSimParamFormFields, simParamsFormFields } from "./formFields";
import {
  PortalRunConfigStringy,
  portalRunConfigStringySchema,
} from "../../config/config";
import { LabelledInputGroup } from "./LabelledInputGroup";

export type FormProps = {
  defaultValues: PortalRunConfigStringy;
  onSubmit: (data: PortalRunConfigStringy) => void;
  onDirtinessChange: (isDirty: boolean) => void;
};

export type FormHandle = {
  getValues: UseFormGetValues<PortalRunConfigStringy>;
  submit: ReturnType<UseFormHandleSubmit<PortalRunConfigStringy>>;
};

export const Form = React.forwardRef(
  (props: FormProps, ref: React.Ref<FormHandle>) => {
    const {
      register,
      handleSubmit,
      watch,
      getValues,
      formState: { errors },
    } = useForm<PortalRunConfigStringy>({
      defaultValues: props.defaultValues,
      resolver: zodResolver(portalRunConfigStringySchema),
    });

    const handledOnSubmit = useCallback(handleSubmit(props.onSubmit), [
      handleSubmit,
      props.onSubmit,
    ]);

    // Built in dirtiness checking had issues when defaultValues changed
    const isDirty = !fp.isEqual(props.defaultValues)(watch());
    useEffect(() => {
      props.onDirtinessChange(isDirty);
    }, [isDirty]);

    useImperativeHandle(ref, () => ({
      getValues,
      submit: handledOnSubmit,
    }));

    return (
      <form onSubmit={handledOnSubmit}>
        {/* Hidden submit to make submit on enter work */}
        <input type="submit" className="invisible absolute" />
        <Collapsible title="Simulation Parameters" defaultExpanded>
          <LabelledInputGroup
            register={register}
            errors={errors}
            fields={simParamsFormFields}
          />
        </Collapsible>
        <Collapsible title="Advanced Simulation Parameters">
          <LabelledInputGroup
            register={register}
            errors={errors}
            fields={advSimParamFormFields}
          />
        </Collapsible>
        <Collapsible title="Data Collection">
          <InfoBox>
            CSV export and desired statistics must be enabled <i>before</i>{" "}
            running simulations
          </InfoBox>
          Some more content here
        </Collapsible>
      </form>
    );
  }
);
