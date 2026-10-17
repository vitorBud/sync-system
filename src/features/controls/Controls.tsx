import { useState } from 'react';
import { native } from '../../platform/window';
import type { ActionId, Capability, Device } from '../../core/devices';
import { SettingsEditor } from './SettingsEditor';
import { ShutdownDialog } from './ShutdownDialog';
import { useControlCenter } from './useControlCenter';

const labels: Record<ActionId, string> = {
  wake: 'Enviar despertar', shutdown: 'Desligar este PC',
  open_remote_play: 'Abrir Remote Play', open_sunshine: 'Abrir Sunshine',
};

export function Controls() {
  const { catalog, settings, busy, error, feedback, controlCenter } = useControlCenter();
  const [editing, setEditing] = useState(false);
  const [shutdown, setShutdown] = useState<Device | null>(null);

  function execute(device: Device, capability: Capability) {
    controlCenter.clearMessages();
    if (capability.confirmationRequired) { setShutdown(device); return; }
    void controlCenter.execute({ deviceId: device.id, actionId: capability.actionId });
  }
  return <div className="controls">
    <div className="section-heading"><span>SEUS DISPOSITIVOS</span>
      <div className="section-tools">
        <button disabled={busy !== null} onClick={() => void controlCenter.refresh()}>{busy === 'refresh' ? 'Verificando…' : 'Atualizar'}</button>
        <button disabled={!native || !settings || busy !== null} onClick={() => setEditing(!editing)}>Configurar rede</button>
      </div>
    </div>
    {error && <p className="control-feedback" role="alert">{error}</p>}
    {feedback && <p className="control-feedback" role="status">{feedback}</p>}
    {editing && settings && <SettingsEditor settings={settings} busy={busy !== null}
      save={value => controlCenter.saveSettings(value)} close={() => setEditing(false)}/>}
    {!catalog && <p className="control-feedback" role="status">{busy ? 'Verificando este computador e seus aplicativos…' : 'Sem verificação atual. Tente Atualizar.'}</p>}
    {catalog?.devices.map(device => <section key={device.id} className="card device-card">
      <div className="device-heading"><h2>{device.name}</h2>
        <span className={'device-badge' + (device.status === 'Online' || device.status === 'Sunshine disponível' || device.status === 'Sessão em uso' ? ' available' : '')}>{device.status}</span></div>
      <p>{device.detail}</p>
      <div className="device-actions">{device.capabilities.map(capability =>
        <button key={capability.actionId} disabled={busy !== null || !capability.enabled} title={capability.reason ?? undefined}
          onClick={() => execute(device, capability)}>{busy === capability.actionId ? 'Executando…' : labels[capability.actionId]}</button>)}
        {device.id === 'gamer_wake' && <button disabled={!native || busy !== null || !settings} onClick={() => setEditing(true)}>Configurar destino</button>}
      </div>
    </section>)}
    {catalog && catalog.checkedAt > 0 && <small className="checked-at">Verificado às {new Date(catalog.checkedAt).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })}</small>}
    <p className="deferred-feature">Iluminação do quarto fica para a última etapa.</p>
    {shutdown && <ShutdownDialog machine={shutdown.detail} busy={busy !== null} error={error} cancel={() => setShutdown(null)}
      confirm={() => { void controlCenter.execute({ deviceId: shutdown.id, actionId: 'shutdown', confirmed: true })
        .then(success => { if (success) setShutdown(null); }); }}/>}
  </div>;
}