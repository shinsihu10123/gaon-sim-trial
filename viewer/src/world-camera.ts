import * as THREE from "three";
import type { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { RenderWorldBounds } from "./types";
import { WORLD_DISPLAY_SIZE, WorldSpaceTransform } from "./world-space";

const MIN_CAMERA_DISTANCE = 3;
const MAX_CAMERA_DISTANCE = WORLD_DISPLAY_SIZE * 2.25;
const CAMERA_TARGET_PADDING = 4;
const MIN_TARGET_Y = -8;
const MAX_TARGET_Y = 12;

export class WorldCameraController {
  private transform: WorldSpaceTransform | undefined;

  public constructor(
    private readonly camera: THREE.PerspectiveCamera,
    private readonly controls: OrbitControls,
  ) {
    controls.enableDamping = true;
    controls.enableZoom = true;
    controls.enableRotate = true;
    controls.enablePan = true;
    controls.screenSpacePanning = true;
    controls.minDistance = MIN_CAMERA_DISTANCE;
    controls.maxDistance = MAX_CAMERA_DISTANCE;
    controls.minPolarAngle = 0.08;
    controls.maxPolarAngle = Math.PI * 0.49;
    controls.keyPanSpeed = 14;
    controls.listenToKeyEvents(document.body);

    camera.near = 0.05;
    camera.far = WORLD_DISPLAY_SIZE * 8;
    camera.updateProjectionMatrix();
  }

  public setWorldBounds(bounds: RenderWorldBounds | null): void {
    this.transform = bounds === null ? undefined : new WorldSpaceTransform(bounds);
    if (this.transform !== undefined) {
      this.frameWorld();
    }
  }

  public frameWorld(): void {
    if (this.transform === undefined) {
      return;
    }
    const radius = Math.max(
      this.transform.halfSceneWidth,
      this.transform.halfSceneDepth,
      WORLD_DISPLAY_SIZE / 4,
    );
    const target = new THREE.Vector3(0, 0, 0);
    const distance = THREE.MathUtils.clamp(radius * 1.75, 24, MAX_CAMERA_DISTANCE * 0.85);
    const direction = new THREE.Vector3(1, 0.78, 1).normalize();
    this.controls.target.copy(target);
    this.camera.position.copy(target).addScaledVector(direction, distance);
    this.camera.lookAt(target);
    this.controls.update();
  }

  public focusWorldPoint(xM: number, zM: number, sceneY = 0): void {
    if (this.transform === undefined) {
      return;
    }
    this.focusScenePoint(this.transform.worldPointToScene(xM, zM, sceneY));
  }

  public focusScenePoint(point: THREE.Vector3): void {
    const offset = this.camera.position.clone().sub(this.controls.target);
    const currentDistance = THREE.MathUtils.clamp(
      Math.max(offset.length(), WORLD_DISPLAY_SIZE * 0.18),
      MIN_CAMERA_DISTANCE,
      WORLD_DISPLAY_SIZE * 0.72,
    );
    if (offset.lengthSq() < Number.EPSILON) {
      offset.set(1, 0.7, 1);
    }
    offset.normalize().multiplyScalar(currentDistance);
    this.controls.target.copy(point);
    this.clampTarget();
    this.camera.position.copy(this.controls.target).add(offset);
    this.camera.lookAt(this.controls.target);
    this.controls.update();
  }

  public update(): void {
    this.clampTarget();
    this.controls.update();
  }

  public dispose(): void {
    this.controls.stopListenToKeyEvents();
    this.controls.dispose();
  }

  private clampTarget(): void {
    if (this.transform === undefined) {
      return;
    }
    this.transform.clampSceneTarget(this.controls.target, CAMERA_TARGET_PADDING);
    this.controls.target.y = THREE.MathUtils.clamp(
      this.controls.target.y,
      MIN_TARGET_Y,
      MAX_TARGET_Y,
    );
  }
}
