import {Document} from "./types";

const DocumentsUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/workshop_literature/search";

export async function SearchDocuments(year: number, vehicle: string, aposNumber: string): Promise<Document[]> {
  let url = DocumentsUrl;

  let body = {
    year: year,
    vehicle: vehicle,
    aposNumber: aposNumber,
    documentType: "RL"
  }

  let response = await fetch(url, {
    method: 'POST',
    headers: {
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(body)
  });

  if (!response.ok) {
    throw new Error('Something went getting the document list');
  }

  let js = await response.json();
  return js.map((doc: Document) => new Document(doc))
}
