import {
  FieldError,
  FieldErrors,
  FieldPath,
  FieldValues,
} from "react-hook-form";
import lodash from "lodash";

export const extractError = <T extends FieldValues>(
  errors: FieldErrors<T>,
  configPath: FieldPath<T>
) => lodash.get(errors, configPath) as unknown as FieldError;
