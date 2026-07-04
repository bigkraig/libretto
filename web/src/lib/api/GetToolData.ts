import {ToolData} from "@/lib/api/types";

const ToolDataUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/content/tool_data";

export async function GetToolData(year: number, vehicle: string, toolId: string): Promise<ToolData> {
  let response = await fetch(ToolDataUrl + "/" + year + "/" + vehicle + "/" + toolId);
  if (!response.ok) {
    throw new Error('Something went getting the tool data');
  }
  return new ToolData(await response.json())
}
