export interface KartaNode {
    id: string;
    ntype: string; // Node type, e.g., 'text', 'image'
    x: number;
    y: number;
    // ... more node properties will be added later
}

export interface KartaEdge {
    id: string;
    source: string;
    target: string;
}
