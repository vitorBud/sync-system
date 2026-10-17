import type { ActionRequest, Catalog, ControlTransport, Settings } from './devices.ts';

export interface ControlState {
  catalog: Catalog | null;
  settings: Settings | null;
  busy: string | null;
  error: string;
  feedback: string;
}
export class ControlCenter {
  private state: ControlState = { catalog: null, settings: null, busy: null, error: '', feedback: '' };
  private listeners = new Set<() => void>();
  private transport: ControlTransport;
  constructor(transport: ControlTransport) { this.transport = transport; }
  getSnapshot = () => this.state;
  subscribe = (listener: () => void) => {
    this.listeners.add(listener);
    return () => { this.listeners.delete(listener); };
  };
  private update(patch: Partial<ControlState>) {
    this.state = { ...this.state, ...patch };
    this.listeners.forEach(listener => listener());
  }
  private async readSnapshot() {
    try {
      const [catalog, settings] = await Promise.all([
        this.transport.readCatalog(), this.transport.readSettings(),
      ]);
      this.update({ catalog, settings, error: '' });
    } catch (error) {
      // A failed check must not leave an old online indicator looking current.
      this.update({ catalog: null, error: String(error instanceof Error ? error.message : error) });
    }
  }
  clearMessages() { this.update({ error: '', feedback: '' }); }
  async refresh() {
    if (this.state.busy) return;
    this.update({ busy: 'refresh', catalog: null, error: '', feedback: '' });
    try { await this.readSnapshot(); }
    finally { this.update({ busy: null }); }
  }
  async saveSettings(settings: Settings) {
    if (this.state.busy) return false;
    this.update({ busy: 'settings', error: '', feedback: '' });
    try {
      await this.transport.saveSettings(settings);
      await this.readSnapshot();
      this.update({ feedback: 'Configurações salvas neste computador.' });
      return true;
    } catch (error) {
      this.update({ error: String(error instanceof Error ? error.message : error) });
      return false;
    } finally { this.update({ busy: null }); }
  }
  async execute(request: ActionRequest) {
    if (this.state.busy) return false;
    const device = this.state.catalog?.devices.find(item => item.id === request.deviceId);
    const capability = device?.capabilities.find(item => item.actionId === request.actionId);
    if (!capability?.enabled || (capability.confirmationRequired && !request.confirmed)) {
      this.update({ error: capability?.reason || 'Ação indisponível ou aguardando confirmação.', feedback: '' });
      return false;
    }
    this.update({ busy: request.actionId, error: '', feedback: '' });
    try {
      const result = await this.transport.execute(request);
      this.update({ feedback: result.message });
      if (request.actionId !== 'shutdown') await this.readSnapshot();
      return true;
    } catch (error) {
      this.update({ error: String(error instanceof Error ? error.message : error) });
      return false;
    } finally { this.update({ busy: null }); }
  }
}