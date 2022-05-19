import React, { useEffect, useRef } from "react";

import { Button } from "./utils/Button";
import * as icons from "./icons/Icons";
import {
  PortalRunConfig,
  SimStatus,
  encodeConfigInURL,
} from "../config/config";
import ClipboardJS from "clipboard";

export type ButtonFooterProps = {
  status: SimStatus;
  config?: PortalRunConfig;
  configIsDirty: boolean;
  startOrRestartSim: () => void;
  pauseSim: () => void;
  resumeSim: () => void;
};

export const ButtonFooter = ({
  status,
  config,
  configIsDirty,
  startOrRestartSim,
  pauseSim,
  resumeSim,
}: ButtonFooterProps) => {
  const started = status != "notStarted";
  const running = status == "running";
  const paused = status == "paused";

  const copySeedRef = useRef<HTMLButtonElement>(null);
  const copyNoSeedRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (config && copySeedRef.current && copyNoSeedRef.current) {
      new ClipboardJS(copySeedRef.current, {
        text: () => encodeConfigInURL(config, true),
      });
      new ClipboardJS(copyNoSeedRef.current, {
        text: () => encodeConfigInURL(config, false),
      });
    }
  }, [config, copySeedRef.current, copyNoSeedRef.current]);

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
        <button disabled={!started} ref={copySeedRef}>
          <icons.LinkSeed className="h-10" />
        </button>
        <button disabled={!started} ref={copyNoSeedRef}>
          <icons.LinkNoSeed className="h-10" />
        </button>
      </div>
    </div>
  );
};
