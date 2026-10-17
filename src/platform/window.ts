import { isTauri, invoke } from '@tauri-apps/api/core';
export const native = isTauri();
export async function resizePanel(expanded: boolean, height?: number) {
  if (native) await invoke('resize_panel', { expanded, height });
}
export async function dragWindow() {
  if (native) {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().startDragging();
  }
}
export async function closeWindow() {
  if (native) {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().close();
  }
}