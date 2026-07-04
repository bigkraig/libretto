const TranslationsUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/content/translations";

export async function GetTranslations(): Promise<Map<string, string>> {
  let response = await fetch(TranslationsUrl);
  if (!response.ok) {
    throw new Error('Something went getting the translations');
  }

  let map: Map<string, string> = new Map<string, string>();
  let js = await response.json();
  for (let member in js) {
    map.set(member, js[member]);
  }

  map.set("LBL_APOS_NUMBER", "Labor operation number");
  map.set("LBL_INUMBER", "I-No.");
  return map;
}
