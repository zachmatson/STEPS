import React from "react";

import { Button } from "./utils/Button";
import * as icons from "./icons/Icons";
import { SimStatus } from "../config/config";

export type ButtonFooterProps = {
  status: SimStatus;
  configIsDirty: boolean;
  startOrRestartSim: () => void;
  pauseSim: () => void;
  resumeSim: () => void;
};

export const ButtonFooter = ({
  status,
  configIsDirty,
  startOrRestartSim,
  pauseSim,
  resumeSim,
}: ButtonFooterProps) => {
  const started = status != "notStarted";
  const running = status == "running";
  const paused = status == "paused";

  return (
    <div className="flex justify-between flex-wrap gap-4 p-2 lg:py-4 border-gray-300 border-t-2">
      <div className="flex items-center gap-1">
        <Button
          color="green"
          infoText={configIsDirty ? "Settings Changed" : undefined}
          onClick={startOrRestartSim}
        >
          {running || paused ? "Restart" : "Run"}
        </Button>
        {running && (
          <Button color="gray" onClick={pauseSim}>
            Pause
          </Button>
        )}
        {paused && (
          <Button color="gray" onClick={resumeSim}>
            Resume
          </Button>
        )}
      </div>

      <div className="flex items-center flex-wrap gap-4">
        <button disabled>
          <icons.Download className="h-10" />
        </button>
        <button disabled={!started}>
          <icons.LinkSeed className="h-10" />
        </button>
        <button disabled={!started}>
          <icons.LinkNoSeed className="h-10" />
        </button>
      </div>
    </div>
  );
};
