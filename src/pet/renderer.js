import { loadAllSprites, getSpriteKey, getSprite } from './sprites.js';

const FPS_MAP = { idle: 2, working: 4, sleeping: 2, compacting: 6, waiting: 3 };

export class PetRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.sessions = [];
    this.animationTime = 0;
    this.lastTimestamp = 0;
    this.spritesLoaded = false;
    this.resize();
    window.addEventListener('resize', () => this.resize());
  }

  resize() {
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = this.canvas.offsetWidth * dpr;
    this.canvas.height = this.canvas.offsetHeight * dpr;
    this.ctx.imageSmoothingEnabled = false;
  }

  async startAnimation() {
    await loadAllSprites();
    this.spritesLoaded = true;
    this.lastTimestamp = performance.now();
    requestAnimationFrame((t) => this.animate(t));
  }

  update(sessions) {
    this.sessions = sessions;
  }

  animate(timestamp) {
    const dt = (timestamp - this.lastTimestamp) / 1000;
    this.lastTimestamp = timestamp;
    this.animationTime += dt;
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
    if (this.spritesLoaded) this.drawSessions();
    requestAnimationFrame((t) => this.animate(t));
  }

  drawSessions() {
    const dpr = window.devicePixelRatio || 1;
    const count = this.sessions.length || 1;
    const spacing = this.canvas.width / (count + 1);

    if (this.sessions.length === 0) {
      this.drawSprite({ task_state: 'idle', emotion: 'neutral', position_index: 0 },
        this.canvas.width / 2, this.canvas.height - 60 * dpr);
      return;
    }

    this.sessions.forEach((session, i) => {
      const x = spacing * (i + 1);
      const y = this.canvas.height - 60 * dpr;
      this.drawSprite(session, x, y);
    });
  }

  drawSprite(session, x, y) {
    const dpr = window.devicePixelRatio || 1;
    const state = session.task_state || 'idle';
    const emotion = session.emotion || 'neutral';
    const key = getSpriteKey(state, emotion);
    if (!key) return;
    const sprite = getSprite(key);
    if (!sprite) return;

    const fps = FPS_MAP[state] || 2;
    const frameIndex = Math.floor(this.animationTime * fps) % sprite.frameCount;
    const sx = frameIndex * sprite.frameWidth;
    const bobOffset = Math.sin(this.animationTime * 1.5 + (session.position_index || 0)) * 3;
    const scale = 2 * dpr;
    const drawW = sprite.frameWidth * scale;
    const drawH = sprite.frameHeight * scale;

    this.ctx.drawImage(sprite.image,
      sx, 0, sprite.frameWidth, sprite.frameHeight,
      x - drawW / 2, y - drawH + bobOffset * dpr, drawW, drawH);
  }
}
