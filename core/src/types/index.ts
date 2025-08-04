export type NodeId = string;
export type ContextId = string;
export type EdgeId = string;

export interface DataNode {
  id: NodeId;
  ntype: string;
  path: string;
  attributes: Record<string, unknown>;
  createdAt: number;
  modifiedAt: number;
}

export interface ViewNode {
  x: number;
  y: number;
  scale: number;
  rotation: number;
  z?: number;
  opacity?: number;
  view_isNameVisible?: boolean;
}

export interface Context {
  id: ContextId;
  viewNodes: Record<NodeId, ViewNode>;
}

export interface Edge {
  id: EdgeId;
  from: NodeId;
  to: NodeId;
  edgeType: string;
  attributes: Record<string, unknown>;
}

export interface KartaBundle {
  version: number;
  exportedAt: string;
  metadata?: {
    title?: string;
    description?: string;
    author?: string;
    minRuntimeVersion?: string;
  };
  nodes: DataNode[];
  contexts: Record<NodeId, { viewNodes: Record<NodeId, ViewNode> }>;
  edges: Edge[];
  assets?: Record<string, unknown>;
}