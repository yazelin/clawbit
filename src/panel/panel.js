const { invoke } = window.__TAURI__.core;

export class PanelUI {
  constructor() {
    this.el = document.getElementById('panel');
    this.visible = false;
    this.activeTab = 'sessions';
  }

  toggle() {
    this.visible = !this.visible;
    this.el.classList.toggle('hidden', !this.visible);
    if (this.visible) this.render();
  }

  async render() {
    const [sessions, stats, settings] = await Promise.all([
      invoke('get_sessions'), invoke('get_stats'), invoke('get_settings'),
    ]);

    this.el.innerHTML = `
      <div class="panel-tabs">
        <button class="tab ${this.activeTab === 'sessions' ? 'active' : ''}" data-tab="sessions">Sessions</button>
        <button class="tab ${this.activeTab === 'stats' ? 'active' : ''}" data-tab="stats">Stats</button>
        <button class="tab ${this.activeTab === 'settings' ? 'active' : ''}" data-tab="settings">Settings</button>
      </div>
      <div class="panel-body">${this.renderTab(sessions, stats, settings)}</div>
    `;

    this.el.querySelectorAll('.tab').forEach(btn => {
      btn.addEventListener('click', () => { this.activeTab = btn.dataset.tab; this.render(); });
    });
    this.bindSettings(settings);
  }

  renderTab(sessions, stats, settings) {
    if (this.activeTab === 'sessions') return this.renderSessions(sessions);
    if (this.activeTab === 'stats') return this.renderStats(stats);
    return this.renderSettings(settings);
  }

  renderSessions(sessions) {
    if (!sessions.length) return '<p class="empty">No active sessions. Start Claude Code to see your Clawbit!</p>';
    return sessions.map(s => `
      <div class="session-item">
        <span class="session-state state-${s.task_state}">${s.task_state}</span>
        <span class="session-emotion">${s.emotion}</span>
        <span class="session-cwd">${s.cwd || '—'}</span>
        <span class="session-stats">${s.turns || 0} turns, ${s.tool_calls || 0} tools</span>
      </div>
    `).join('');
  }

  renderStats(stats) {
    return `<div class="stats-grid">
      <div class="stat"><span class="stat-value">${stats.total_sessions}</span><span class="stat-label">Sessions</span></div>
      <div class="stat"><span class="stat-value">${stats.total_turns}</span><span class="stat-label">Turns</span></div>
      <div class="stat"><span class="stat-value">${stats.total_tool_calls}</span><span class="stat-label">Tool Calls</span></div>
    </div>`;
  }

  renderSettings(settings) {
    return `<div class="settings-list">
      <label class="setting"><span>Hooks</span>
        <button id="btn-hooks">${settings.hooks_installed ? 'Uninstall' : 'Install'}</button></label>
      <label class="setting"><span>Sound</span>
        <input type="checkbox" id="chk-mute" ${settings.muted ? '' : 'checked'}></label>
      <label class="setting"><span>Theme</span>
        <select id="sel-theme">
          <option value="system" ${settings.theme === 'system' ? 'selected' : ''}>System</option>
          <option value="light" ${settings.theme === 'light' ? 'selected' : ''}>Light</option>
          <option value="dark" ${settings.theme === 'dark' ? 'selected' : ''}>Dark</option>
        </select></label>
      <label class="setting"><span>Autostart</span>
        <input type="checkbox" id="chk-autostart" ${settings.autostart ? 'checked' : ''}></label>
    </div>`;
  }

  bindSettings(settings) {
    document.getElementById('btn-hooks')?.addEventListener('click', async () => {
      if (settings.hooks_installed) { await invoke('uninstall_hooks'); settings.hooks_installed = false; }
      else { await invoke('install_hooks'); settings.hooks_installed = true; }
      await invoke('update_settings', { newSettings: settings });
      this.render();
    });

    document.getElementById('sel-theme')?.addEventListener('change', async (e) => {
      settings.theme = e.target.value;
      document.body.dataset.theme = settings.theme === 'system'
        ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
        : settings.theme;
      await invoke('update_settings', { newSettings: settings });
    });

    document.getElementById('chk-mute')?.addEventListener('change', async (e) => {
      settings.muted = !e.target.checked;
      await invoke('update_settings', { newSettings: settings });
    });

    document.getElementById('chk-autostart')?.addEventListener('change', async (e) => {
      settings.autostart = e.target.checked;
      await invoke('update_settings', { newSettings: settings });
    });
  }

  updateSessions(sessions) {
    if (this.visible && this.activeTab === 'sessions') this.render();
  }
}
