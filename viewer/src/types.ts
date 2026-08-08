export type RegionSurface = "land" | "ocean";

export interface RenderDate {
  year: number;
  month: number;
  day: number;
}

export interface RenderWorldBounds {
  minXM: number;
  maxXM: number;
  minZM: number;
  maxZM: number;
}

export interface RenderTerrainSnapshot {
  width: number;
  height: number;
  spacingM: number;
  seaLevelM: number;
  elevationM: number[];
  moisturePermille: number[];
  reliefCodes: number[];
  biomeCodes: number[];
  downstreamIndices: number[];
  drainageBasinIds: number[];
  flowAccumulation: number[];
  riverOrders: number[];
  landmassIds: number[];
  islandLandmassIds: number[];
}

export interface RenderMapPoint {
  xM: number;
  zM: number;
}

export interface RenderRegionSnapshot {
  id: string;
  surface: RegionSurface;
  center: RenderMapPoint;
  boundary: RenderMapPoint[];
  neighbors: string[];
  legalOwner: string | null;
  controller: string | null;
}

export interface RenderIdentitySnapshot {
  id: string;
}

export interface RenderCitySnapshot {
  id: string;
  countryId: string | null;
  regionId: string | null;
}

export interface RenderHumanGroupSnapshot {
  id: string;
  regionId: string;
  xM: number;
  zM: number;
  population: string;
  foodStockPersonDays: string;
  basicResourceStockUnits: string;
  mobilityPermille: number;
  explorationPermille: number;
  settlementBiasPermille: number;
}

export interface RenderWorldSnapshot {
  initialized: boolean;
  bounds: RenderWorldBounds | null;
  terrain: RenderTerrainSnapshot | null;
  regions: RenderRegionSnapshot[];
  humanGroups: RenderHumanGroupSnapshot[];
  settlements: RenderIdentitySnapshot[];
  communities: RenderIdentitySnapshot[];
  politicalEntities: RenderIdentitySnapshot[];
  countries: RenderIdentitySnapshot[];
  cities: RenderCitySnapshot[];
}

export interface RenderSnapshot {
  version: number;
  date: RenderDate;
  elapsedDays: number;
  commandCount: number;
  eventCount: number;
  authoritativeDigestHex: string;
  world: RenderWorldSnapshot;
}
