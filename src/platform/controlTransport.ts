import { invoke } from '@tauri-apps/api/core';
import { native } from './window';
import type { ActionResult, Catalog, ControlTransport, Settings } from '../core/devices';

const preview: Catalog = {
  checkedAt: 0,
  devices: [
    { id: 'local_pc', name: 'Este computador', status: 'Prévia', detail: 'Abra o Sync System desktop para consultar este computador.', capabilities: [] },
    { id: 'gamer_wake', name: 'Acordar PC gamer', status: 'Prévia', detail: 'Configure o destino no aplicativo desktop.', capabilities: [] },
    { id: 'switch_streaming', name: 'Streaming para Switch', status: 'Prévia', detail: 'Disponibilidade do Sunshine será consultada no aplicativo desktop.', capabilities: [] },
    { id: 'ps5', name: 'PlayStation 5', status: 'Prévia', detail: 'O aplicativo desktop pode abrir o PS Remote Play instalado.', capabilities: [] },
  ],
};
export const controlTransport: ControlTransport = {
  async readCatalog() {
    return native ? invoke<Catalog>('device_catalog') : preview;
  },
  async readSettings() {
    return native ? invoke<Settings>('control_settings') : { version: 1, wakeTarget: null, sunshinePort: 47989 };
  },
  async saveSettings(settings) {
    if (!native) throw new Error('Use o aplicativo desktop para salvar a configuração.');
    await invoke('save_control_settings', { settings });
  },
  async execute(request) {
    if (!native) throw new Error('Use o aplicativo desktop para executar esta ação.');
    return invoke<ActionResult>('execute_action', { request });
  },
};