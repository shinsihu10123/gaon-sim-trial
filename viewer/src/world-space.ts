import * as THREE from "three";
import type { RenderWorldBounds } from "./types";

export const WORLD_DISPLAY_SIZE = 80;
export const VERTICAL_EXAGGERATION = 8;

export class WorldSpaceTransform {
  public readonly centerXM: number;
  public readonly centerZM: number;
  public readonly extentXM: number;
  public readonly extentZM: number;
  public readonly horizontalScale: number;
  public readonly verticalScale: number;
  public readonly halfSceneWidth: number;
  public readonly halfSceneDepth: number;

  public constructor(public readonly bounds: RenderWorldBounds) {
    this.centerXM = (bounds.minXM + bounds.maxXM) / 2;
    this.centerZM = (bounds.minZM + bounds.maxZM) / 2;
    this.extentXM = Math.max(bounds.maxXM - bounds.minXM, 1);
    this.extentZM = Math.max(bounds.maxZM - bounds.minZM, 1);
    this.horizontalScale = WORLD_DISPLAY_SIZE / Math.max(this.extentXM, this.extentZM);
    this.verticalScale = this.horizontalScale * VERTICAL_EXAGGERATION;
    this.halfSceneWidth = (this.extentXM * this.horizontalScale) / 2;
    this.halfSceneDepth = (this.extentZM * this.horizontalScale) / 2;
  }

  public worldPointToScene(xM: number, zM: number, sceneY = 0): THREE.Vector3 {
    return new THREE.Vector3(
      (xM - this.centerXM) * this.horizontalScale,
      sceneY,
      (zM - this.centerZM) * this.horizontalScale,
    );
  }

  public elevationToSceneY(elevationM: number): number {
    return elevationM * this.verticalScale;
  }

  public clampSceneTarget(target: THREE.Vector3, padding = 0): THREE.Vector3 {
    target.x = THREE.MathUtils.clamp(
      target.x,
      -this.halfSceneWidth - padding,
      this.halfSceneWidth + padding,
    );
    target.z = THREE.MathUtils.clamp(
      target.z,
      -this.halfSceneDepth - padding,
      this.halfSceneDepth + padding,
    );
    return target;
  }
}
