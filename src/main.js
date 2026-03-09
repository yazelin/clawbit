const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

import { PetRenderer } from './pet/renderer.js';
import { PanelUI } from './panel/panel.js';

const canvas = document.getElementById('pet-canvas');
const renderer = new PetRenderer(canvas);
const panel = new PanelUI();

// Drag support — entire container is draggable except grass island
const appWindow = getCurrentWindow();
document.getElementById('pet-container').addEventListener('mousedown', async (e) => {
  if (e.target.id === 'grass-island') return;
  if (e.target.closest('#panel')) return;
  await appWindow.startDragging();
});

// Grass island click → toggle panel
document.getElementById('grass-island').addEventListener('click', () => {
  panel.toggle();
});

// Listen for pet updates from backend
listen('pet-update', async () => {
  const sessions = await invoke('get_sessions');
  renderer.update(sessions);
  panel.updateSessions(sessions);
});

// Initial load
async function init() {
  const settings = await invoke('get_settings');
  const theme = settings.theme === 'system'
    ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
    : settings.theme;
  document.body.dataset.theme = theme;

  const sessions = await invoke('get_sessions');
  renderer.update(sessions);
  renderer.startAnimation();
}

init();
