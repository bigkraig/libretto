const IllustrationsUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicle_component_tree/illustrations";

export async function GetIllustration(id: number): Promise<string> {
  const url = IllustrationsUrl + "/" + id;
  let response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Something went getting the illustration from ${url}`);
  }
  return response.text()
}
