import { writable, derived, get, type Readable } from "svelte/store";
import type { DataNode, NodeId } from "../types";

export class BaseNodeStore {
  protected _nodes = writable<Map<NodeId, DataNode>>(new Map());

  get nodes(): Readable<Map<NodeId, DataNode>> {
    return derived(this._nodes, (n) => n);
  }

  getNode(id: NodeId): DataNode | undefined {
    return get(this._nodes).get(id);
  }

  protected setNode(node: DataNode): void {
    this._nodes.update((nodes) => {
      nodes.set(node.id, node);
      return nodes;
    });
  }
}