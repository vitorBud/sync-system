import { useState } from 'react';
import type { Settings } from '../../core/devices';

function initialSettings(settings: Settings) {
  if (settings.wakeTarget) return settings;
  try {
    const legacy = JSON.parse(localStorage.getItem('sync.wake') ?? 'null');
    if (typeof legacy?.mac === 'string' && typeof legacy?.broadcast === 'string') {
      return { ...settings, wakeTarget: { mac: legacy.mac, broadcast: legacy.broadcast } };
    }
  } catch { /* Keep an unconfigured target. */ }
  return settings;
}
export function SettingsEditor({ settings, busy, save, close }: {
  settings: Settings; busy: boolean; save: (settings: Settings) => Promise<boolean>; close: () => void;
}) {
  const [draft, setDraft] = useState(() => initialSettings(settings));
  const [wakeEnabled, setWakeEnabled] = useState(draft.wakeTarget !== null);
  const [mac, setMac] = useState(draft.wakeTarget?.mac ?? '');
  const [broadcast, setBroadcast] = useState(draft.wakeTarget?.broadcast ?? '255.255.255.255');
  return <section className="card device-card">
    <h2>Rede deste computador</h2>
    <form className="device-settings" onSubmit={e => {
      e.preventDefault();
      void save({ ...draft, wakeTarget: wakeEnabled ? { mac: mac.trim(), broadcast: broadcast.trim() } : null })
        .then(saved => { if (saved) close(); });
    }}>
      <label className="check-label"><input type="checkbox" checked={wakeEnabled} disabled={busy} onChange={e => setWakeEnabled(e.target.checked)}/>Configurar despertar do PC gamer</label>
      {wakeEnabled && <>
        <label>MAC da placa de rede do PC gamer<input required disabled={busy} value={mac} placeholder="02:11:22:33:44:55" onChange={e => setMac(e.target.value)}/></label>
        <label>Broadcast da rede<input required disabled={busy} value={broadcast} onChange={e => setBroadcast(e.target.value)}/></label>
        <small>Esses dados identificam quem receberá o despertar. Use este controle a partir de outro dispositivo ligado.</small>
      </>}
      <label>Porta do Sunshine neste PC<input type="number" required min={1024} max={65534} disabled={busy}
        value={draft.sunshinePort} onChange={e => setDraft({ ...draft, sunshinePort: Number(e.target.value) })}/></label>
      <small>Use 47989 se você não alterou a porta nas configurações do Sunshine. Os dados são salvos somente neste computador.</small>
      <div className="device-actions"><button disabled={busy} type="submit">Salvar configuração</button>
        <button disabled={busy} type="button" onClick={close}>Cancelar</button></div>
    </form>
  </section>;
}