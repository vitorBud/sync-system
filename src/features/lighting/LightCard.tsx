import { useEffect, useRef, useState } from 'react';
import { native } from '../../platform/window';
import {
  connectTuya, disconnectTuya, getLightStatus, hasTuyaConnection, openTuyaGuide, setLightPower,
  type LightStatus, type TuyaConfig,
} from './api';

function emptyDraft(): TuyaConfig {
  return { region: '', clientId: '', clientSecret: '', deviceId: '', relayCode: '' };
}
export function LightCard() {
  const [configured, setConfigured] = useState(false);
  const [status, setStatus] = useState<LightStatus | null>(null);
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(emptyDraft);
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [forgetting, setForgetting] = useState(false);
  const inFlight = useRef(false);

  useEffect(() => {
    let active = true;
    if (native) {
      inFlight.current = true;
      setBusy(true);
      void (async () => {
        try {
          const exists = await hasTuyaConnection();
          if (!active) return;
          setConfigured(exists);
          if (exists) {
            const snapshot = await getLightStatus();
            if (active) setStatus(snapshot);
          }
        } catch {
          if (active) setMessage('Não foi possível consultar a conexão. Tente conectar ou atualizar novamente.');
        } finally {
          if (active) { inFlight.current = false; setBusy(false); }
        }
      })();
    }
    return () => { active = false; };
  }, []);
  async function run(action: () => Promise<void>) {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setMessage('');
    try { await action(); }
    catch (error) {
      setStatus(null);
      setMessage(String(error instanceof Error ? error.message : error));
    } finally { setBusy(false); inFlight.current = false; }
  }

  async function save(e: React.FormEvent) {
    e.preventDefault();
    await run(async () => {
      const snapshot = await connectTuya({
        region: draft.region,
        clientId: draft.clientId.trim(),
        clientSecret: draft.clientSecret.trim(),
        deviceId: draft.deviceId.trim(),
        relayCode: draft.relayCode.trim(),
      });
      setStatus(snapshot);
      setConfigured(true);
      setDraft(emptyDraft());
      setEditing(false);
      setMessage(snapshot.online === false ? 'Conexão salva. A tomada está offline na Tuya.' : 'Conexão salva no cofre do sistema. Estado consultado na Tuya.');
    });
  }

  function power(on: boolean) {
    return run(async () => {
      const snapshot = await setLightPower(on);
      setStatus(snapshot);
      setMessage(snapshot.online !== true ? 'A tomada ficou indisponível. Atualize o estado.'
        : snapshot.on === on ? 'Estado da tomada confirmado pela Tuya.'
        : 'Comando aceito; a mudança de estado ainda não foi confirmada. Atualize o estado.');
    });
  }

  const label = !native ? 'Prévia' : !configured ? 'Não conectada'
    : status?.online === false ? 'Offline'
    : status?.online !== true ? 'Não verificado'
    : status.on === null ? 'Online · estado pendente' : status.on ? 'Ligada' : 'Desligada';

  return <section className="card device-card">
    <div className="device-heading"><h2>LED do quarto</h2>
      <span className={'device-badge' + (status?.online ? ' available' : '')}>{label}</span></div>
    <p>{status?.deviceName || 'Tomada inteligente · Smart Life'}</p>
    <div className="device-actions">
      <button disabled={!native || busy || status?.online !== true} onClick={() => void power(true)}>Ligar</button>
      <button disabled={!native || busy || status?.online !== true} onClick={() => void power(false)}>Desligar</button>
      {configured && <button disabled={busy} onClick={() => void run(async () => {
        setStatus(await getLightStatus());
        setMessage('Estado consultado na Tuya.');
      })}>Atualizar estado</button>}
      <button disabled={!native || busy} onClick={() => {
        setDraft(emptyDraft()); setEditing(!editing); setForgetting(false);
      }}>{configured ? 'Configuração' : 'Conectar tomada'}</button>
    </div>
    {busy && <small role="status">Conectando à Tuya…</small>}
    {editing && <form className="device-settings" onSubmit={e => void save(e)}>
      <p>Vincule o Smart Life ao seu projeto Tuya por QR code. Depois preencha as chaves do projeto e o ID da tomada.</p>
      <button type="button" className="text-button setup-link" disabled={busy} onClick={() => void openTuyaGuide().catch(() => setMessage('Não foi possível abrir o guia no navegador.'))}>Guia oficial de configuração Tuya ↗</button>
      <label>Data center do projeto<select required value={draft.region} disabled={busy} onChange={e => setDraft({ ...draft, region: e.target.value })}>
        <option value="">Selecione a região do projeto</option>
        <option value="us">Western America</option><option value="us-east">Eastern America</option>
        <option value="eu">Central Europe</option><option value="eu-west">Western Europe</option>
        <option value="cn">China</option><option value="in">India</option>
      </select></label>
      <label>Client ID / Access ID<input required autoComplete="off" maxLength={96} disabled={busy} value={draft.clientId} onChange={e => setDraft({ ...draft, clientId: e.target.value })}/></label>
      <label>Client Secret / Access Secret<input required type="password" autoComplete="new-password" maxLength={128} minLength={8} disabled={busy} value={draft.clientSecret} onChange={e => setDraft({ ...draft, clientSecret: e.target.value })}/></label>
      <label>Device ID da tomada<input required autoComplete="off" maxLength={96} disabled={busy} value={draft.deviceId} onChange={e => setDraft({ ...draft, deviceId: e.target.value })}/></label>
      <details><summary>Tomada com mais de um canal</summary>
        <label>Canal do interruptor<input autoComplete="off" maxLength={64} disabled={busy} placeholder="Automático para uma única saída" value={draft.relayCode} onChange={e => setDraft({ ...draft, relayCode: e.target.value })}/></label>
      </details>
      <small>Ao conectar, os dados são usados para autenticar na Tuya e consultar esta tomada. As chaves ficam no cofre do sistema. O acesso depende do plano de API do seu projeto.</small>
      <div className="device-actions"><button type="submit" disabled={busy}>Salvar e conectar à Tuya</button><button type="button" disabled={busy} onClick={() => {
        setDraft(emptyDraft()); setEditing(false);
      }}>Cancelar</button></div>
      {configured && <button type="button" className="text-button" disabled={busy} onClick={() => setForgetting(true)}>Remover conexão deste computador</button>}
      {forgetting && <div className="power-confirm">
        <p>Remover as credenciais salvas neste computador? A tomada continuará no Smart Life.</p>
        <div className="device-actions"><button type="button" disabled={busy} onClick={() => setForgetting(false)}>Cancelar remoção</button>
          <button type="button" disabled={busy} onClick={() => void run(async () => {
            await disconnectTuya();
            setStatus(null); setConfigured(false); setEditing(false);
            setForgetting(false); setDraft(emptyDraft()); setMessage('Conexão removida deste computador.');
          })}>Sim, remover conexão</button></div>
      </div>}
    </form>}
    {message && <p className="control-feedback" role="status">{message}</p>}
  </section>;
}