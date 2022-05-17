export type SafeParseResult<T> =
  | {
      success: true;
      data: T;
    }
  | {
      success: false;
    };

export const safeJSONParse = (value: string): SafeParseResult<any> => {
  try {
    return { success: true, data: JSON.parse(value) };
  } catch (e) {
    return { success: false };
  }
};
