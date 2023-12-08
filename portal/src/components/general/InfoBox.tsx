import React from "react";

import * as icons from "../icons/Icons";

export type InfoBoxType = "info" | "success";

export type InfoBoxProps = {
  type?: InfoBoxType;
  children: React.ReactNode;
};

export const InfoBox = ({ type = "info", children }: InfoBoxProps) => {
  let Icon;
  let extraIconClasses = "";
  switch (type) {
    case "info":
      Icon = icons.InfoCircle;
      break;
    case "success":
      Icon = icons.Checkmark;
      extraIconClasses = "text-emerald-600";
      break;
  }
  return (
    <div className="flex items-center rounded-md px-3 py-1 mb-3.5 bg-gray-300 black">
      <span>
        <Icon className={`h-4 select-none ${extraIconClasses}`} />
      </span>
      <span className="flex-auto py-2 pl-2">{children}</span>
    </div>
  );
};
