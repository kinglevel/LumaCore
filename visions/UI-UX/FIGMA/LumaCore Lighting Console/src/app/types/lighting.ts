export type FixtureType = 
  | "Moving Head"
  | "PAR"
  | "Wash"
  | "Strobe"
  | "Laser"
  | "LED Bar";

export interface Fixture {
  id: string;
  name: string;
  type: FixtureType;
  manufacturer: string;
  dmxAddress: number;
  universe: number;
  channels: number;
  intensity: number;
  pan: number;
  tilt: number;
  color: string;
  gobo: number;
  imageUrl?: string;
}

export interface Scene {
  id: string;
  name: string;
  cueNumber: number;
  fadeTime: number;
  fixtureStates: {
    fixtureId: string;
    intensity: number;
    pan?: number;
    tilt?: number;
    color?: string;
    gobo?: number;
  }[];
  createdAt: string;
}

export interface Universe {
  id: number;
  name: string;
  fixtures: Fixture[];
  outputEnabled: boolean;
}
