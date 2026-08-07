import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { SimulationBridge } from "./simulation-bridge";
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
scene.background = new THREE.Color(0x111111);
const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 1000);
camera.position.set(44, 48, 44);
const renderer = new THREE.WebGLRenderer({ antialias: true });
viewport.append(renderer.domElement);
const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;

scene.add(new THREE.HemisphereLight(0xffffff, 0x333333, 1.2));
const plane = new THREE.Mesh(
  new THREE.PlaneGeometry(90, 90),
  new THREE.MeshStandardMaterial({ color: 0x252525, roughness: 1 }),
);
plane.rotation.x = -Math.PI / 2;
scene.add(plane);
scene.add(new THREE.GridHelper(90, 18, 0x555555, 0x333333));

const layer = new WorldTopologyLayer(scene);
const bridge = new SimulationBridge();

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
  controls.update();
  renderer.render(scene, camera);
  requestAnimationFrame(frame);
}
frame();

const snapshot = await bridge.initialize("00000000000007ea");
layer.update(snapshot.world);
panel.textContent = `Year ${snapshot.date.year} · regions ${snapshot.world.regions.length} · ${snapshot.authoritativeDigestHex}`;
