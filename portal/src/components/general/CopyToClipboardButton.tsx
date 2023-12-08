import React, { useCallback, useState } from "react";

import clipboard from "clipboardy";

import { Tooltip } from "./Tooltip";

export type CopyToClipboardButtonProps = {
  getText: () => string;
  tooltipDescription?: string;
  disabled?: boolean;
  children: React.ReactNode;
};

export const CopyToClipboardButton = ({
  getText,
  tooltipDescription,
  disabled,
  children,
}: CopyToClipboardButtonProps) => {
  const onClick = useCallback(async () => {
    await clipboard.write(getText());
    setShowCopiedMessage(true);
  }, [getText]);

  const [showCopiedMessage, setShowCopiedMessage] = useState(false);
  const copiedMessageTooltipOnClose = useCallback(
    () => setShowCopiedMessage(false),
    []
  );

  const buttonElement = (
    <button disabled={disabled} onClick={onClick}>
      {children}
    </button>
  );

  return showCopiedMessage ? (
    <Tooltip
      text="Copied!"
      onClose={copiedMessageTooltipOnClose}
      infoBoxType={"success"}
    >
      {buttonElement}
    </Tooltip>
  ) : (
    <Tooltip text={tooltipDescription}>{buttonElement}</Tooltip>
  );
};
