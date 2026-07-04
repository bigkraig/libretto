const WorkshopImageUrl = process.env.NEXT_PUBLIC_API_IMAGE_HOST + "/v1/content/workshop_image";

export function GetWorkshopImageUrl(imageId: string, size: string): string {
  return WorkshopImageUrl + "/" +  imageId + "/" + size;
}
