import {TreeNode} from "./types";

const TreeNodesUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicle_component_tree";

export async function GetTreeNodes(vehicle: string, year: number, node_id: number | null): Promise<TreeNode> {
  let url = TreeNodesUrl;
  if (node_id != null) {
    url += `/nodes/${node_id}`
  } else {
    url += `/${year}/${vehicle}`
  }

  let response = await fetch(url);
  if (!response.ok) {
    throw new Error('Something went getting the tree nodes');
  }
  let js = await response.json();
  return new TreeNode(js)
}
