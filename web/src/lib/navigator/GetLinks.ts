import {NavigatorLink} from "@/lib/navigator/GetLinks.types";
import {ListVehicles, Vehicle} from "@/lib/api";
import {GetTreeNodes} from "@/lib/api/GetTreeNodes";
import {HREF} from "@/lib";

export const vehicleGlyphs: { [key: string]: string } = {
  "991810": "911",
  "Y1BFH1": "caye",
  "981810": "caym",
  "F151M": "911", // TODO -- its a ferrarrrri
  "R8": "caym",   // Audi R8 -- coupe glyph
}

export async function GetLinks(vehicle: string | null, year: number | null, node_id: number | null): Promise<NavigatorLink[]> {

  // No vehicle or year selected, show the vehicle selector
  if (vehicle == null || year == null) {
    let vehicles = await ListVehicles();
    return vehicles.map((vehicle: Vehicle): NavigatorLink => {
      return {
        location: "000",
        text: `${vehicle.name} (${vehicle.vehicle})`,
        kind: "vehicle",
        href: HREF(vehicle.vehicle, vehicle.year),
        selected: false,
        icon: vehicleGlyphs[vehicle.vehicle],
      }
    });
  }

  let tree_node = await GetTreeNodes(vehicle, year, node_id);
  return NavigatorLink.FromTreeNode(tree_node);
}
