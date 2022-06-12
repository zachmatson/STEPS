import React from "react";

export const genericReactMemo = React.memo as <T>(component: T) => T;
