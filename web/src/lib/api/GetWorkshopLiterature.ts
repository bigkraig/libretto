import {IWorkshopLiterature, WorkshopLiterature} from "./types";

const WorkshopLiteratureUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/workshop_literature";

export async function GetWorkshopLiterature(hkap_id: string): Promise<IWorkshopLiterature> {
  let response = await fetch(WorkshopLiteratureUrl + "/" + hkap_id);
  if (!response.ok) {
    throw new Error('Something went getting the workshop literature');
  }
  let js = await response.json();
  let wl = new WorkshopLiterature(js['payload']);
  return wl
}
