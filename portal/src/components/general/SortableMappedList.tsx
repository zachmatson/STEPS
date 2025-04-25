import React from "react";

import Sortable from "sortablejs";
import { v4 as uuidv4 } from "uuid";

export type SortableMappedListRenderInput<K> = {
  key: K;
  handleClassname?: string;
};

export type SortableMappedListProps<K> = {
  keys: K[];
  handle?: boolean;
  children: (elementProps: SortableMappedListRenderInput<K>) => React.ReactNode;
};

export class SortableMappedList<K> extends React.Component<
  SortableMappedListProps<K>
> {
  sortable: Sortable | null = null;
  containerRef = React.createRef<HTMLDivElement>();
  handleClassname = "handle-" + uuidv4(); // Classname must start with letter

  componentDidMount() {
    if (this.containerRef.current) {
      this.sortable = Sortable.create(this.containerRef.current);
      this.componentDidUpdate();
    }
  }

  componentWillUnmount() {
    this.sortable?.destroy();
  }

  componentDidUpdate() {
    this.sortable?.option(
      "handle",
      this.props.handle ? "." + this.handleClassname : undefined
    );
  }

  render() {
    return (
      <div ref={this.containerRef}>
        {this.props.keys.map((key) =>
          this.props.children({ key, handleClassname: this.handleClassname })
        )}
      </div>
    );
  }
}
