import React, { useRef } from "react";

import { cloneDeep, isEqual } from "lodash";

export const genericReactMemo = React.memo as <T>(component: T) => T;

export const useMatchRefsToVals = <T>(obj: T): T => {
  const lastValueRef = useRef<T>();
  if (!lastValueRef.current || !isEqual(lastValueRef.current, obj)) {
    lastValueRef.current = cloneDeep(obj);
  }
  return lastValueRef.current;
};
