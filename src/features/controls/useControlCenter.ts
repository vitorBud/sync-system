import { useEffect, useSyncExternalStore } from 'react';
import { ControlCenter } from '../../core/controlCenter';
import { controlTransport } from '../../platform/controlTransport';

export const controlCenter = new ControlCenter(controlTransport);
export function useControlCenter() {
  const state = useSyncExternalStore(controlCenter.subscribe, controlCenter.getSnapshot);
  useEffect(() => {
    void controlCenter.refresh();
  }, []);
  return { ...state, controlCenter };
}