import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { EntitySelectionLayer } from "./entity-selection";
import { SpatialEventFocusController, type SpatialFocusEvent } from "./event-focus";
import { HumanGroupLayer } from "./human-group-layer";
import { EntitySelectionPanel } from "./selection-panel";
import { SimulationBridge } from "./simulation-bridge";
import { TerrainLayer } from "./terrain-layer";
import type { RenderSnapshot } from "./types";
import { WorldCameraController } from "./world-camera";
import { WorldTopologyLayer } from "./world-layer";
import "./style.css";

const root = document.querySelector<HTMLDivElement>("#app");
if (!root) throw new Error("missing app root");

const viewport = document.createElement("div");
viewport.className = "viewport";
const panel = document.createElement("aside");
panel.className = "status-panel";
root.append(viewport, panel);

const scene = new THREE.Scene();
scene.background = new THREE.Color(0xb9c7cd);
scene.fog = new THREE.FogExp2(0xb9c7cd, 0.0065);
const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 1000);
camera.position.set(52, 42, 52);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 1.08;
viewport.append(renderer.domElement);

const controls = new OrbitControls(camera, renderer.domElement);
const cameraController = new WorldCameraController(camera, controls);
const eventFocusController = new SpatialEventFocusController(cameraController);

scene.add(new THREE.HemisphereLight(0xeef7ff, 0x44513f, 1.4));
const sun = new THREE.DirectionalLight(0xfff7e8, 2.15);
sun.position.set(35, 60, 20);
scene.add(sun);

const stagingPlane = new THREE.Mesh(
  new THREE.PlaneGeometry(90, 90),
  new THREE.MeshStandardMaterial({ color: 0x252525, roughness: 1 }),
);
stagingPlane.rotation.x = -Math.PI / 2;
scene.add(stagingPlane);
const stagingGrid = new THREE.GridHelper(90, 18, 0x555555, 0x333333);
scene.add(stagingGrid);

const terrainLayer = new TerrainLayer(scene);
const topologyLayer = new WorldTopologyLayer(scene);
const humanGroupLayer = new HumanGroupLayer(scene);
const selectionLayer = new EntitySelectionLayer(scene);
const selectionPanel = new EntitySelectionPanel(root);
const bridge = new SimulationBridge();

selectionPanel.onSelectionChanged = (target) => selectionLayer.select(target);
selectionPanel.onFocusRequested = (target) => {
  selectionLayer.select(target);
  cameraController.focusScenePoint(target.scenePoint);
};
selectionPanel.onWorldRequested = () => cameraController.frameWorld();

let pointerDownX = 0;
let pointerDownY = 0;
renderer.domElement.addEventListener("pointerdown", (event) => {
  pointerDownX = event.clientX;
  pointerDownY = event.clientY;
});
renderer.domElement.addEventListener("pointerup", (event) => {
  if (Math.hypot(event.clientX - pointerDownX, event.clientY - pointerDownY) > 5) return;
  const target = selectionLayer.pickFromClientPoint(event.clientX, event.clientY, camera, renderer.domElement);
  selectionLayer.select(target);
  selectionPanel.setSelected(target);
});
renderer.domElement.addEventListener("dblclick", (event) => {
  const target = selectionLayer.pickFromClientPoint(event.clientX, event.clientY, camera, renderer.domElement);
  if (target !== undefined) {
    selectionLayer.select(target);
    selectionPanel.setSelected(target);
    cameraController.focusScenePoint(target.scenePoint);
  }
});

function resize(): void {
  const width = Math.max(viewport.clientWidth, 1);
  const height = Math.max(viewport.clientHeight, 1);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
  renderer.setSize(width, height, false);
}
window.addEventListener("resize", resize);
resize();

function frame(): void {
  cameraController.update();
  renderer.render(scene, camera);
  requestAnimationFrame(frame);
}
frame();

function applySnapshot(snapshot: RenderSnapshot): void {
  const hasTerrain = snapshot.world.terrain !== null;
  stagingPlane.visible = !hasTerrain;
  stagingGrid.visible = !hasTerrain;
  terrainLayer.update(snapshot.world.bounds, snapshot.world.terrain);
  topologyLayer.update(snapshot.world);
  humanGroupLayer.update(snapshot.world);
  selectionLayer.update(snapshot.world);
  selectionPanel.setTargets(selectionLayer.listTargets());
  const selected = selectionLayer.selectedTarget();
  selectionPanel.setSelected(selected);
  cameraController.setWorldBounds(snapshot.world.bounds);

  const terrain = snapshot.world.terrain;
  panel.textContent = terrain === null
    ? `Year ${snapshot.date.year} · terrain unavailable · ${snapshot.authoritativeDigestHex}`
    : `Year ${snapshot.date.year} · terrain ${terrain.width}×${terrain.height} · regions ${snapshot.world.regions.length} · human groups ${snapshot.world.humanGroups.length} · ${snapshot.authoritativeDigestHex}`;
}

applySnapshot(await bridge.initialize("00000000000007ea"));

export async function advanceViewerDays(days: number): Promise<RenderSnapshot> {
  const snapshot = await bridge.advanceDays(days);
  applySnapshot(snapshot);
  return snapshot;
}

export function autoFocusSpatialEvent(event: SpatialFocusEvent): boolean {
  return eventFocusController.handle(event);
}

window.addEventListener("beforeunload", () => {
  terrainLayer.dispose();
  topologyLayer.dispose();
  humanGroupLayer.dispose();
  selectionLayer.dispose();
  selectionPanel.dispose();
  cameraController.dispose();
  bridge.dispose();
  renderer.dispose();
});
