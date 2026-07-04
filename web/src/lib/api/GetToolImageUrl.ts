const ToolImageUrl = process.env.NEXT_PUBLIC_API_IMAGE_HOST + "/v1/content/tool_image";

export function GetToolImageUrl(imageId: string): string {
  return ToolImageUrl + "/" +  btoa(imageId);
}
