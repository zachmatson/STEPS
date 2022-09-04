import React, { createRef } from "react";

import Popup from "reactjs-popup";
import { PopupActions } from "reactjs-popup/dist/types";

import { InfoBox, InfoBoxType } from "./InfoBox";

export type TooltipProps = {
  text?: string;
  infoBoxType?: InfoBoxType;
  children: React.ReactElement;
  onClose?: () => void;
};

export class Tooltip extends React.Component<TooltipProps> {
  ref = createRef<PopupActions>();
  togglingPopupToForceReposition = false;

  isOpen = false;
  onOpen = () => {
    this.isOpen = true;
  };
  onClose = () => {
    if (this.togglingPopupToForceReposition) {
      this.togglingPopupToForceReposition = false;
      this.ref.current?.open();
      return;
    }
    this.isOpen = false;
    this.props.onClose?.();
  };

  componentDidUpdate(prevProps: Readonly<TooltipProps>) {
    if (this.isOpen && this.popupPositionWillChange(this.props, prevProps)) {
      this.togglingPopupToForceReposition = true;
      this.ref.current?.close();
    }
  }

  render() {
    const { text, children, infoBoxType } = this.props;

    return text ? (
      <Popup
        ref={this.ref}
        trigger={children}
        position={"top center"}
        arrow={false}
        on={["hover", "focus"]}
        mouseEnterDelay={150}
        closeOnDocumentClick={true}
        onOpen={this.onOpen}
        onClose={this.onClose}
      >
        <InfoBox type={infoBoxType}>{text}</InfoBox>
      </Popup>
    ) : (
      children
    );
  }

  popupPositionWillChange(initialProps: TooltipProps, newProps: TooltipProps) {
    return (
      initialProps.text !== newProps.text ||
      initialProps.infoBoxType !== newProps.infoBoxType
    );
  }
}
