import { FieldErrors, FieldPath } from "react-hook-form";
import lodash from "lodash";

export const extractError = <T>(
  errors: FieldErrors<T>,
  configPath: FieldPath<T>
) => lodash.get(errors, configPath);
