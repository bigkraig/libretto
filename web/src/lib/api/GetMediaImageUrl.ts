const MediaImageUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/content/media";

export function GetMediaImageUrl(imageId: string): string {
  return MediaImageUrl + "/" +  imageId;
}
