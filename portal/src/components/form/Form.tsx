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

import { Collapsible } from "../utils/Collapsible";
import { InfoBox } from "../utils/InfoBox";
import {
  advSimParamFormFields,
  FormFieldSet,
  simParamsFormFields,
} from "./formFields";
import {
  portalRunConfigSchema,
  PortalRunConfigStringy,
} from "../../config/config";
import { TextInputGroup } from "./TextInputGroup";
import { useMatchRefsToVals } from "../../utils/useMatchRefsToVals";

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
        <Collapsible title="Data Collection">
          <InfoBox>
            CSV export and desired statistics must be enabled <i>before</i>{" "}
            running simulations
          </InfoBox>
          <div className="pb-3.5 pl-0.5">
            <label className="cursor-pointer flex items-center">
              <input
                type="checkbox"
                className={
                  "appearance-none w-5 h-5 border-gray-300 border-2 rounded mr-2 cursor-pointer bg-gray-50 \
                   flex justify-center items-center \
                   checked:before:h-3 checked:before:w-3 checked:before:rounded-sm checked:before:bg-blue-600"
                }
              />
              <span className="select-none">Enable Download</span>
            </label>
          </div>
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
