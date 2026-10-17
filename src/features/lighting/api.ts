import { invoke } from '@tauri-apps/api/core';
import { native } from '../../platform/window';

export interface TuyaConfig {
  region: string;
  clientId: string;
  clientSecret: string;
  deviceId: string;
  relayCode: string;
}
export interface LightStatus {
  configured: boolean;
  online: boolean | null;
  on: boolean | null;
  deviceName: string | null;
  relayCode: string | null;
}
function requireNative() {
  if (!native) throw new Error('Use o Sync System desktop para conectar sua tomada.');
}
export async function hasTuyaConnection() {
  requireNative();
  return invoke<boolean>('tuya_connection');
}
export async function connectTuya(config: TuyaConfig) {
  requireNative();
  return invoke<LightStatus>('connect_tuya', { config });
}
export async function getLightStatus() {
  requireNative();
  return invoke<LightStatus>('tuya_status');
}
export async function setLightPower(on: boolean) {
  requireNative();
  return invoke<LightStatus>('set_tuya_power', { on });
}
export async function disconnectTuya() {
  requireNative();
  await invoke('disconnect_tuya');
}
export async function openTuyaGuide() {
  requireNative();
  await invoke('open_tuya_guide');
}