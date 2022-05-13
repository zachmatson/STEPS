import React from "react";

import * as icons from "./Icons";

export type InfoBoxProps = {
  children: React.ReactNode;
};

export const InfoBox = ({ children }: InfoBoxProps) => (
  <div className="flex items-center rounded-md p-3 bg-gray-300 black">
    <span>
      <icons.InfoCircle className="h-4 select-none" />
    </span>
    <span className="flex-auto pl-2">{children}</span>
  </div>
);
