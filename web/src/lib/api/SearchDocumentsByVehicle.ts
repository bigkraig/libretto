import {Document} from "./types";

const BaseUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicles/";

export async function SearchDocumentsByVehicle(year: number, vehicle: string, query: string, signal?: AbortSignal): Promise<Document[]> {
  const url = `${BaseUrl}${year}/${vehicle}/documents/search?q=${encodeURIComponent(query)}`;

  const response = await fetch(url, {signal});
  if (!response.ok) {
    throw new Error('Failed to search documents');
  }
  const js = await response.json();
  return js.map((doc: Document) => new Document(doc));
}
