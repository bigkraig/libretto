import {IVehicle, Vehicle} from "./types";

const VehiclesUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicles";

export async function ListVehicles(): Promise<Vehicle[]> {
  let response = await fetch(VehiclesUrl);
  if (!response.ok) {
    throw new Error('Something went getting the vehicles');
  }
  return (await response.json()).map((e: IVehicle) => new Vehicle(e)).sort((n1: IVehicle, n2: IVehicle) => {
    if (n1.name > n2.name) {
      return 1;
    }
    if (n1.name < n2.name) {
      return -1;
    }
    return 0;
  });
}