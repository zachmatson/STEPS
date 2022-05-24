import React from "react";

import * as icons from "../icons/Icons";

export type InfoBoxProps = {
  children: React.ReactNode;
};

export const InfoBox = ({ children }: InfoBoxProps) => (
  <div className="flex items-center rounded-md px-3 py-1 mb-3.5 bg-gray-300 black">
    <span>
      <icons.InfoCircle className="h-4 select-none" />
    </span>
    <span className="flex-auto py-2 pl-2">{children}</span>
  </div>
);
