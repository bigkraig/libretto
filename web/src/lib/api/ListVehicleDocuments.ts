import {Document} from "./types";

const Base = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicle_component_tree";

// Every document for the vehicle (root subtree) — shown before a component is picked.
export async function ListVehicleDocuments(year: number, vehicle: string): Promise<Document[]> {
  const res = await fetch(`${Base}/${year}/${vehicle}/documents`);
  if (!res.ok) {
    throw new Error('Something went wrong getting the vehicle document list');
  }
  const js = await res.json();
  return js.map((doc: Document) => new Document(doc));
}
