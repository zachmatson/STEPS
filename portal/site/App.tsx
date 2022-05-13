import React from "react";

import { Collapsible } from "./utils/Collapsible";
import { InfoBox } from "./utils/InfoBox";
import * as icons from "./utils/Icons";
import { ChartTest } from "./utils/ChartTest";

export const App = () => (
  <div className="flex justify-center h-full">
    <div className="flex flex-col justify-start items-center h-full w-full max-w-full 2xl:max-w-[1536px]">
      <header className="w-full py-3 lg:py-6 flex justify-center items-center border-gray-300 border-b-2 lg:border-0 text-3xl select-none cursor-default">
        STEPS Portal
      </header>
      <main className="w-full flex-auto flex flex-col lg:flex-row lg:justify-center">
        <div className="lg:w-1/3 pb-2 lg:pb-0 px-0">
          <Collapsible title="Simulation Parameters" defaultExpanded>
            <div className="pb-2 pl-0.5">Some content</div>
            <input type="text" className="w-full max-w-sm" />
          </Collapsible>
          <Collapsible title="Advanced Simulation Parameters">
            <div>Some more content</div>
            <button type="button" className="button button-blue">
              Here's a Button
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
              CSV export and desired statistics must be enabled <i>before</i>{" "}
              running simulations
            </InfoBox>
            <ChartTest />
          </Collapsible>
        </div>
        <div className="lg:w-2/3 px-0 flex-auto lg:h-full">
          <div className="flex justify-center items-center text-gray-500 h-full select-none cursor-default">
            Run Simulations to See Data
          </div>
        </div>
      </main>
      <footer className="w-full max-w-[inherit] fixed bottom-0 flex justify-between flex-wrap gap-2 p-2 lg:py-4 border-gray-300 border-t-2">
        <div>
          <button type="button" className="button button-green">
            Run
          </button>{" "}
          <button type="button" className="button button-gray">
            Pause
          </button>
        </div>
        <div className="h-full flex items-center">
          <button disabled>
            <icons.Download className="h-10 px-2" />
          </button>
          <button>
            <icons.LinkSeed className="h-10 px-2" />
          </button>
          <button>
            <icons.LinkNoSeed className="h-10 px-2" />
          </button>
        </div>
      </footer>
    </div>
  </div>
);
