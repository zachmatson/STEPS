import { z } from "zod";

const requiredErrorMessage = "Field reqiuired";
const shouldBeU64ErrorMessage = "Field should be a 64-bit unsigned integer";
const shouldBeBooleanErrorMessage = "Field should be a boolean";

const numParser =
  (regex?: RegExp) =>
  (value: unknown): any => {
    if (value === "") return undefined;
    if (regex && typeof value === "string" && !regex.test(value)) return "";
    const attempt = Number(value);
    return isNaN(attempt) ? value : attempt;
  };

const bigIntParser = (value: unknown): any => {
  if (value === "") return undefined;
  if (typeof value === "string" && /^\d+$/.test(value)) return BigInt(value);
  if (typeof value === "number" && value % 1 == 0) return BigInt(value);
  return value;
};

const zNumberFactory = <T extends z.ZodType>(
  invalidMsg: string,
  postTransform: (schema: z.ZodNumber, msg: string) => T,
  preParseTest?: RegExp
) => {
  const innerResult = postTransform(
    z.number({
      required_error: requiredErrorMessage,
      invalid_type_error: invalidMsg,
    }),
    invalidMsg
  );

  return z.preprocess(numParser(preParseTest), innerResult);
};

/**
 * Extends z.optional() a bit by converting empty strings to undefined
 * Needed to deal with strangeness of forms where `undefined` gets coerced to `""`
 */
export const zOptional = <T extends z.ZodType>(inner: T): z.ZodOptional<T> => {
  return z.preprocess(
    (value) => (value == "" ? undefined : value),
    z.optional(inner)
  ) as unknown as z.ZodOptional<T>;
};

export const zPosNumber = zNumberFactory(
  "Field should be a positive number",
  (schema, msg) => schema.positive(msg)
);

export const zNonNegNumber = zNumberFactory(
  "Field should be a non-negative number",
  (schema, msg) => schema.nonnegative(msg)
);

export const zGe1Number = zNumberFactory(
  "Field should be a number and at least 1",
  (schema, msg) => schema.min(1, msg)
);

export const zPosInt = zNumberFactory(
  "Field should be a positive integer",
  (schema, msg) => schema.positive(msg).int(msg),
  /^\d+$/
);

export const zSeed = z.preprocess(
  bigIntParser,
  z
    .bigint({
      required_error: requiredErrorMessage,
      invalid_type_error: shouldBeU64ErrorMessage,
    })
    .refine((x) => x >= 0 && x < 2n ** 64n, {
      message: shouldBeU64ErrorMessage,
    })
    .default(() => BigInt(Math.floor(Math.random() * 2 ** 64 - 1)))
);

export const zBoolean = z.boolean({
  required_error: requiredErrorMessage,
  invalid_type_error: shouldBeBooleanErrorMessage,
});

export const zString = z.preprocess(
  String,
  z.string({
    required_error: requiredErrorMessage,
  })
);
