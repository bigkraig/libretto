import {Vehicle} from "./types";

const VehiclesUrl = process.env.NEXT_PUBLIC_API_HOST + "/v1/vehicles";

export async function GetVehicle(vehicle: string, year: number): Promise<Vehicle> {
  let response = await fetch(VehiclesUrl + "/" + year + "/" + vehicle);
  if (!response.ok) {
    throw new Error('Something went getting the vehicle');
  }
  return new Vehicle(await response.json())
}
