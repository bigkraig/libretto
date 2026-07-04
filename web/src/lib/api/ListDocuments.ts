import {Document} from "./types";

const DocumentsUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicle_component_tree/";

export async function ListDocuments(node_id: number): Promise<Document[]> {
  let url = DocumentsUrl + `nodes/${node_id}/documents`;

  let response = await fetch(url);
  if (!response.ok) {
    throw new Error('Something went getting the document list');
  }
  let js = await response.json();
  return js.map((doc: Document) => new Document(doc))
}
