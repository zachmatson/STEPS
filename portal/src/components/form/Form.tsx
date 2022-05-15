import { Collapsible } from "../utils/Collapsible";
import { InfoBox } from "../utils/InfoBox";
import React from "react";

export const Form = () => (
  <div className="overflow-y-auto">
    <Collapsible title="Simulation Parameters" defaultExpanded>
      <div className="pb-2 pl-0.5">Some content</div>
      <input type="text" className="w-full max-w-sm" />
    </Collapsible>
    <Collapsible title="Advanced Simulation Parameters">
      <div>Some more content</div>
      <button type="button" className="button button-blue">
        {"Here's a Button"}
      </button>{" "}
      <button
        type="button"
        className="button button-gray"
        onClick={() => alert("HI")}
      >
        And Another
      </button>
    </Collapsible>
    <Collapsible title="Data Collection">
      <InfoBox>
        CSV export and desired statistics must be enabled <i>before</i> running
        simulations
      </InfoBox>
      Some more content here
    </Collapsible>
  </div>
);
