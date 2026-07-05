const Base = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicle_component_tree";

export interface AncestorNode {
  node_id: number;
  node_value: string;
  name: string | null;
}

// Ancestor path root -> current for a node, in one request (backend recursive CTE).
export async function GetAncestors(nodeId: number): Promise<AncestorNode[]> {
  const res = await fetch(`${Base}/nodes/${nodeId}/ancestors`);
  if (!res.ok) {
    throw new Error('Something went wrong getting the node ancestors');
  }
  return res.json();
}
