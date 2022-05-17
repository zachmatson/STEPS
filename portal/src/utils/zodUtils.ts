import { z } from "zod";

const requiredErrorMessage = "Field reqiuired";
const shouldBeNumberErrorMessage = "Field should be a number";
const shouldBeIntegerErrorMessage = "Field should be an integer";
const shouldBeU64ErrorMessage = "Field should be a 64-bit unsigned integer";
const shouldBeBooleanErrorMessage = "Field should be a boolean";

export const zNumber = z.number({
  required_error: requiredErrorMessage,
  invalid_type_error: shouldBeNumberErrorMessage,
});

export const zInt = zNumber.int({
  message: shouldBeIntegerErrorMessage,
});

export const zU64 = z
  .bigint({
    required_error: requiredErrorMessage,
    invalid_type_error: shouldBeU64ErrorMessage,
  })
  .refine((x) => x < 2n ** 64n - 1n, {
    message: shouldBeU64ErrorMessage
  });

export const zBoolean = z.boolean({
  required_error: requiredErrorMessage,
  invalid_type_error: shouldBeBooleanErrorMessage,
});

export const zString = z.string({
  required_error: requiredErrorMessage,
});
