import React, { useEffect, useRef } from "react";

import { cloneDeep, isEqual } from "lodash";
import { Observable } from "rxjs";

export const genericReactMemo = React.memo as <T>(component: T) => T;

export const useMatchRefsToVals = <T>(obj: T): T => {
  const lastValueRef = useRef<T>();
  if (!lastValueRef.current || !isEqual(lastValueRef.current, obj)) {
    lastValueRef.current = cloneDeep(obj);
  }
  return lastValueRef.current;
};

export const useSubscription = <T>(
  observable: Observable<T>,
  consumer: (value: T) => void
) => {
  useEffect(() => {
    const subscription = observable.subscribe(consumer);
    return () => {
      subscription.unsubscribe();
    };
  }, [observable, consumer]);
};
