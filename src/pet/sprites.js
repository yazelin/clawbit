const STATES = ['idle', 'working', 'sleeping', 'compacting', 'waiting'];
const EMOTIONS = ['neutral', 'happy', 'sad', 'sob'];
const FRAMES_PER_SHEET = 6;
const cache = new Map();

export async function loadAllSprites() {
  for (const state of STATES) {
    for (const emotion of EMOTIONS) {
      const key = `${state}_${emotion}`;
      const img = new Image();
      img.src = `assets/sprites/${key}.jpg`;
      try {
        await new Promise((resolve, reject) => {
          img.onload = resolve;
          img.onerror = reject;
        });
        cache.set(key, {
          image: img,
          frameCount: FRAMES_PER_SHEET,
          frameWidth: img.width / FRAMES_PER_SHEET,
          frameHeight: img.height,
        });
      } catch { /* Sprite not available, will use fallback */ }
    }
  }
}

export function getSpriteKey(state, emotion) {
  const key = `${state}_${emotion}`;
  if (cache.has(key)) return key;
  // Fallback chain
  const fallbacks = [`${state}_neutral`, 'idle_neutral'];
  for (const fb of fallbacks) {
    if (cache.has(fb)) return fb;
  }
  return null;
}

export function getSprite(key) {
  return cache.get(key);
}

export { FRAMES_PER_SHEET };
