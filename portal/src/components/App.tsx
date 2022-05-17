import React from "react";

import fp from "lodash/fp";

import { PageLayout } from "./PageLayout";
import { Form, FormHandle } from "./form/Form";
import { ButtonFooter } from "./ButtonFooter";
import { NoResultsPlaceholder } from "./results/NoResultsPlaceholder";
import {
  defaultPortalRunConfig,
  PortalRunConfig,
  portalRunConfigSchema,
  PortalRunConfigStringy,
  portalRunConfigStringySchema,
  SimStatus,
} from "../config/config";

type AppState = {
  status: SimStatus;
  activeConfig?: PortalRunConfig;
  activeConfigStringy: PortalRunConfigStringy;
  configIsDirty: boolean;
};

export class App extends React.Component<{}, AppState> {
  constructor(props: {}) {
    super(props);

    let activeConfigStringy: PortalRunConfigStringy = defaultPortalRunConfig;
    const url = new URL(window.location.href);
    if (url.searchParams.has("runConfig")) {
      try {
        const suppliedConfig = JSON.parse(
          atob(url.searchParams.get("runConfig")!)
        );
        activeConfigStringy =
          portalRunConfigStringySchema.parse(suppliedConfig);
      } catch (e) {
        console.log("An invalid configuration was provided in the URL");
      }
    }

    this.state = {
      status: "notStarted",
      activeConfigStringy,
      configIsDirty: false,
    };
  }

  formRef = React.createRef<FormHandle>();

  startOrRestartSim = () => {
    this.formRef.current?.submit();
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

  setConfigIsDirty = (isDirty: boolean) => {
    this.setState({
      configIsDirty: isDirty,
    });
  };

  onValidatedFormSubmit = () => {
    // The data argument to this function would give us the transformed data,
    // this gives us the raw data we want
    const data = fp.cloneDeep(this.formRef.current?.getValues());
    if (!data) return;

    this.setState(
      {
        status: "running",
        activeConfigStringy: data,
        activeConfig: portalRunConfigSchema.parse(data),
        configIsDirty: false,
      },
      () => {
        setTimeout(() => this.setState({ status: "finished" }), 1300);
      }
    );
  };

  render() {
    const form = (
      <Form
        ref={this.formRef}
        defaultValues={this.state.activeConfigStringy}
        onSubmit={this.onValidatedFormSubmit}
        onDirtinessChange={this.setConfigIsDirty}
      />
    );
    const results = <NoResultsPlaceholder />;
    const footer = (
      <ButtonFooter
        status={this.state.status}
        configIsDirty={this.state.configIsDirty}
        startOrRestartSim={this.startOrRestartSim}
        pauseSim={this.pauseSim}
        resumeSim={this.resumeSim}
      />
    );

    return (
      <PageLayout
        {...{
          form,
          results,
          footer,
        }}
      />
    );
  }
}
