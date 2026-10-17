export type DeviceId = 'local_pc' | 'gamer_wake' | 'switch_streaming' | 'ps5';
export type ActionId = 'wake' | 'shutdown' | 'open_remote_play' | 'open_sunshine';
export interface Capability {
  actionId: ActionId;
  enabled: boolean;
  reason: string | null;
  confirmationRequired: boolean;
}
export interface Device {
  id: DeviceId;
  name: string;
  status: string;
  detail: string;
  capabilities: Capability[];
}
export interface Catalog { devices: Device[]; checkedAt: number }
export interface WakeTarget { mac: string; broadcast: string }
export interface Settings { version: 1; wakeTarget: WakeTarget | null; sunshinePort: number }
export interface ActionRequest { deviceId: DeviceId; actionId: ActionId; confirmed?: boolean }
export interface ActionResult { message: string }
export interface ControlTransport {
  readCatalog(): Promise<Catalog>;
  readSettings(): Promise<Settings>;
  saveSettings(settings: Settings): Promise<void>;
  execute(request: ActionRequest): Promise<ActionResult>;
}