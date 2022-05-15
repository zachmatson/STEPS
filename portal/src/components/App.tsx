import React from "react";

import { PageLayout } from "./PageLayout";
import { Form } from "./form/Form";
import { ButtonFooter } from "./ButtonFooter";
import { NoResultsPlaceholder } from "./results/NoResultsPlaceholder";
import { SimStatus } from "../simulations/simulations";

type AppState = {
  status: SimStatus;
  dirtyConfig: boolean;
};

export class App extends React.Component<{}, AppState> {
  constructor(props: {}) {
    super(props);
    this.state = {
      status: "notStarted",
      dirtyConfig: false,
    };
  }

  startOrRestartSim = () => {
    this.setState({ status: "running" });
    setTimeout(() => this.setState({ status: "finished" }), 2000);
  };

  pauseSim = () => {
    if (this.state.status == "running") {
      this.setState({ status: "paused" });
    }
  };

  resumeSim = () => {
    if (this.state.status == "paused") {
      this.setState({ status: "running" });
    }
  };

  render() {
    return (
      <PageLayout
        bodyLeft={<Form />}
        bodyRight={<NoResultsPlaceholder />}
        footer={
          <ButtonFooter
            status={this.state.status}
            dirtyConfig={this.state.dirtyConfig}
            startOrRestartSim={this.startOrRestartSim}
            pauseSim={this.pauseSim}
            resumeSim={this.resumeSim}
          />
        }
      />
    );
  }
}
