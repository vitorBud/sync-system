import { useState } from 'react';
export function Notes() {
  const [note, setNote] = useState(() => {
    try { return localStorage.getItem('sync.note') ?? ''; } catch { return ''; }
  });
  const [failed, setFailed] = useState(false);
  return <section className="card notes"><label htmlFor="note">Uma coisa para lembrar</label>
    <textarea id="note" placeholder="Tire uma ideia da cabeça…" value={note} onChange={e => {
      setNote(e.target.value);
      try { localStorage.setItem('sync.note', e.target.value); setFailed(false); } catch { setFailed(true); }
    }} /><small role="status">{failed ? 'Não foi possível salvar neste dispositivo.' : 'Salvo apenas neste dispositivo'}</small></section>;
}