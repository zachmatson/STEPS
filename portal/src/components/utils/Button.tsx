import React from "react";

import * as icons from "../icons/Icons";

// import "./Button.css";

export type ButtonProps = {
  children: string;
  color: "green" | "gray" | "blue";
  infoText?: string;
  onClick?: React.MouseEventHandler<HTMLButtonElement>;
};

export const Button = React.memo(
  ({ children, color, infoText, onClick }: ButtonProps) => {
    let colorClass: string;
    switch (color) {
      case "gray":
        colorClass = "bg-gray-600 group-hover:bg-gray-700";
        break;
      case "green":
        colorClass = "bg-green-600 group-hover:bg-green-700";
        break;
      case "blue":
        colorClass = "bg-blue-600 group-hover:bg-blue-700";
        break;
    }

    const classes = `py-2 px-3 select-none border-none text-white \
      ${infoText ? "rounded-l" : "rounded"} \
      ${colorClass}`;

    return (
      <button type="button" className="group" onClick={onClick}>
        <span className="flex items-center">
          <span className={`${classes}`}>{children}</span>
          {infoText && (
            <span className="rounded-r flex items-center pl-2 pr-3 text-black bg-gray-300 group-hover:bg-gray-600 group-hover:text-white">
              <icons.InfoCircle className="h-4" />
              <span className="py-2 pl-1">{infoText}</span>
            </span>
          )}
        </span>
      </button>
    );
  }
);
