import * as THREE from "three";
import type { RenderTerrainSnapshot, RenderWorldBounds } from "./types";

const WORLD_DISPLAY_SIZE = 80;
const VERTICAL_EXAGGERATION = 8;

export class TerrainLayer {
  private terrainMesh: THREE.Mesh | undefined;
  private seaMesh: THREE.Mesh | undefined;

  public constructor(private readonly scene: THREE.Scene) {}

  public update(bounds: RenderWorldBounds | null, terrain: RenderTerrainSnapshot | null): void {
    this.clear();
    if (bounds === null || terrain === null) {
      return;
    }

    const sampleCount = terrain.width * terrain.height;
    if (
      terrain.elevationM.length !== sampleCount ||
      terrain.biomeCodes.length !== sampleCount ||
      terrain.reliefCodes.length !== sampleCount
    ) {
      throw new Error("terrain snapshot arrays do not match grid dimensions");
    }

    const extentX = bounds.maxXM - bounds.minXM;
    const extentZ = bounds.maxZM - bounds.minZM;
    const horizontalScale = WORLD_DISPLAY_SIZE / Math.max(extentX, extentZ, 1);
    const verticalScale = horizontalScale * VERTICAL_EXAGGERATION;
    const centerX = (bounds.minXM + bounds.maxXM) / 2;
    const centerZ = (bounds.minZM + bounds.maxZM) / 2;

    const positions = new Float32Array(sampleCount * 3);
    const colors = new Float32Array(sampleCount * 3);
    const color = new THREE.Color();

    for (let z = 0; z < terrain.height; z += 1) {
      for (let x = 0; x < terrain.width; x += 1) {
        const index = z * terrain.width + x;
        const offset = index * 3;
        const xM = bounds.minXM + x * terrain.spacingM;
        const zM = bounds.minZM + z * terrain.spacingM;

        positions[offset] = (xM - centerX) * horizontalScale;
        positions[offset + 1] = terrain.elevationM[index] * verticalScale;
        positions[offset + 2] = (zM - centerZ) * horizontalScale;

        color.setHex(biomeColor(terrain.biomeCodes[index], terrain.reliefCodes[index]));
        colors[offset] = color.r;
        colors[offset + 1] = color.g;
        colors[offset + 2] = color.b;
      }
    }

    const indices: number[] = [];
    for (let z = 0; z < terrain.height - 1; z += 1) {
      for (let x = 0; x < terrain.width - 1; x += 1) {
        const northWest = z * terrain.width + x;
        const northEast = northWest + 1;
        const southWest = northWest + terrain.width;
        const southEast = southWest + 1;
        indices.push(northWest, southWest, northEast, northEast, southWest, southEast);
      }
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    geometry.setAttribute("color", new THREE.BufferAttribute(colors, 3));
    geometry.setIndex(indices);
    geometry.computeVertexNormals();

    const material = new THREE.MeshStandardMaterial({
      vertexColors: true,
      roughness: 0.92,
      metalness: 0,
    });
    this.terrainMesh = new THREE.Mesh(geometry, material);
    this.terrainMesh.name = "authoritative-terrain-heightfield";
    this.scene.add(this.terrainMesh);

    const seaWidth = extentX * horizontalScale;
    const seaHeight = extentZ * horizontalScale;
    const seaGeometry = new THREE.PlaneGeometry(seaWidth, seaHeight);
    const seaMaterial = new THREE.MeshStandardMaterial({
      color: 0x315b74,
      transparent: true,
      opacity: 0.72,
      roughness: 0.35,
      metalness: 0.05,
      side: THREE.DoubleSide,
    });
    this.seaMesh = new THREE.Mesh(seaGeometry, seaMaterial);
    this.seaMesh.rotation.x = -Math.PI / 2;
    this.seaMesh.position.y = terrain.seaLevelM * verticalScale + 0.015;
    this.seaMesh.name = "authoritative-sea-level";
    this.scene.add(this.seaMesh);
  }

  public dispose(): void {
    this.clear();
  }

  private clear(): void {
    if (this.terrainMesh !== undefined) {
      this.terrainMesh.removeFromParent();
      this.terrainMesh.geometry.dispose();
      disposeMaterial(this.terrainMesh.material);
      this.terrainMesh = undefined;
    }
    if (this.seaMesh !== undefined) {
      this.seaMesh.removeFromParent();
      this.seaMesh.geometry.dispose();
      disposeMaterial(this.seaMesh.material);
      this.seaMesh = undefined;
    }
  }
}

function disposeMaterial(material: THREE.Material | THREE.Material[]): void {
  if (Array.isArray(material)) {
    for (const item of material) {
      item.dispose();
    }
  } else {
    material.dispose();
  }
}

function biomeColor(biomeCode: number, reliefCode: number): number {
  switch (biomeCode) {
    case 0:
      return reliefCode === 0 ? 0x17364a : 0x27556f;
    case 1:
      return 0x70835a;
    case 2:
      return 0x3f6146;
    case 3:
      return 0xa69468;
    case 4:
      return 0x586f5e;
    case 5:
      return 0xaeb0ad;
    default:
      return 0x777777;
  }
}
