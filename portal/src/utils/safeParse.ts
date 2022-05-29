import { Base64 } from "js-base64";

export type SafeParseResult<T> =
  | {
      success: true;
      data: T;
    }
  | {
      success: false;
    };

const safeParse =
  (parser: (value: string) => string) =>
  (value: string): SafeParseResult<any> => {
    try {
      return { success: true, data: parser(value) };
    } catch (e) {
      return { success: false };
    }
  };

export const safeJSONParse = safeParse(JSON.parse);
export const safeBase64Decode = safeParse(Base64.decode);
