import {NavigatorLink} from "@/lib/navigator/GetLinks.types";
import {ListVehicles, Vehicle} from "@/lib/api";
import {GetTreeNodes} from "@/lib/api/GetTreeNodes";
import {HREF} from "@/lib";

// Marque per vehicle code. The badge renders /marques/<marque>.svg (drop official
// emblem SVGs there); a monogram stands in until one is present.
export const vehicleMarque: { [key: string]: string } = {
  "991810": "porsche",
  "981810": "porsche",
  "Y1BFH1": "porsche",
  "F151M": "ferrari",
  "R8": "audi",
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
        icon: vehicleMarque[vehicle.vehicle],
      }
    });
  }

  let tree_node = await GetTreeNodes(vehicle, year, node_id);
  return NavigatorLink.FromTreeNode(tree_node);
}
