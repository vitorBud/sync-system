import { useEffect, useRef } from 'react';

export function ShutdownDialog({ machine, busy, error, confirm, cancel }: {
  machine: string; busy: boolean; error: string; confirm: () => void; cancel: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => { element?.close(); };
  }, []);
  return <dialog ref={dialog} className="shutdown-dialog" aria-labelledby="shutdown-title" aria-describedby="shutdown-description"
    onKeyDown={e => { if (e.key === 'Escape') e.stopPropagation(); }}
    onCancel={e => { e.preventDefault(); if (!busy) cancel(); }}>
    <h2 id="shutdown-title">Desligar este computador?</h2>
    <p id="shutdown-description">{machine}. Salve seu trabalho antes de continuar. Esta ação encerra o Windows deste computador.</p>
    {error && <p className="control-feedback" role="alert">{error}</p>}
    <div className="device-actions"><button autoFocus disabled={busy} onClick={cancel}>Cancelar</button>
      <button className="danger-action" disabled={busy} onClick={confirm}>Sim, desligar este PC</button></div>
  </dialog>;
}