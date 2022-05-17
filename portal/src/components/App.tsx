import React from "react";

import { PageLayout } from "./PageLayout";
import { Form, FormHandle } from "./form/Form";
import { ButtonFooter } from "./ButtonFooter";
import { NoResultsPlaceholder } from "./results/NoResultsPlaceholder";
import {
  decodeConfigFromURL,
  defaultPortalRunConfig,
  PortalRunConfig,
  portalRunConfigSchema,
  PortalRunConfigStringy,
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

    // Load from URL
    let activeConfigStringy: PortalRunConfigStringy = defaultPortalRunConfig;
    const suppliedConfig = decodeConfigFromURL();
    if (suppliedConfig.success) {
      activeConfigStringy = suppliedConfig.data;
    }

    this.state = {
      status: "notStarted",
      activeConfigStringy,
      configIsDirty: false,
    };
  }

  formRef = React.createRef<FormHandle>();
  formKey = 0;

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
    // Reset form, React-Hook-Form behaves weirdly after submission
    // We want a successful submission to act like a new form with new defaults
    this.formKey += 1;

    // Use getValue instead because we want the stringy data here
    // This submit handler gets a data argument too but it is the wrong type,
    // TypeScript doesn't seem to understand that either
    const data = this.formRef.current?.getValues();
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
    return (
      <PageLayout
        form={
          <Form
            key={`form-${this.formKey}`}
            ref={this.formRef}
            defaultValues={this.state.activeConfigStringy}
            onSubmit={this.onValidatedFormSubmit}
            onDirtinessChange={this.setConfigIsDirty}
          />
        }
        results={<NoResultsPlaceholder />}
        footer={
          <ButtonFooter
            status={this.state.status}
            config={this.state.activeConfig}
            configIsDirty={this.state.configIsDirty}
            startOrRestartSim={this.startOrRestartSim}
            pauseSim={this.pauseSim}
            resumeSim={this.resumeSim}
          />
        }
      />
    );
  }
}
