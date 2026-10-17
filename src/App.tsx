import { useEffect, useRef, useState } from 'react';
import { flushSync } from 'react-dom';
import { Notes } from './features/notes/Notes';
import { closeWindow, dragWindow, native, resizePanel } from './platform/window';

export function App() {
  const [expanded, setExpanded] = useState(false);
  const [focus, setFocus] = useState(false);
  const [error, setError] = useState('');
  const toggleRef = useRef<HTMLButtonElement>(null);
  const busy = useRef(false);
  const panelRef = useRef<HTMLElement>(null);
  const shellRef = useRef<HTMLElement>(null);

  useEffect(() => {
    document.documentElement.dataset.runtime = native ? 'native' : 'browser';
  }, []);
  async function toggle(next: boolean) {
    if (busy.current || next === expanded) return;
    busy.current = true;
    let animation: Animation | undefined;
    try {
      // Reserve space before revealing; shrink only after the exit finishes.
      if (next) {
        flushSync(() => setExpanded(true));
        // Measure the actual layout before revealing it in the native window.
        const panel = panelRef.current;
        if (panel) panel.style.visibility = 'hidden';
        try {
          await resizePanel(true, Math.ceil(shellRef.current?.getBoundingClientRect().height ?? 580));
        } catch (error) {
          flushSync(() => setExpanded(false));
          throw error;
        } finally {
          if (panel) panel.style.removeProperty('visibility');
        }
      } else {
        toggleRef.current?.focus();
      }
      if (!window.matchMedia('(prefers-reduced-motion: reduce)').matches && panelRef.current) {
        const collapsed = { opacity: 0, transform: 'translateY(-10px) scale(0.97)' };
        const open = { opacity: 1, transform: 'translateY(0) scale(1)' };
        animation = panelRef.current.animate(next ? [collapsed, open] : [open, collapsed], {
          duration: next ? 320 : 220,
          easing: next ? 'cubic-bezier(0.16, 1, 0.3, 1)' : 'cubic-bezier(0.4, 0, 1, 1)',
          fill: 'forwards',
        });
        await animation.finished;
      }
      if (!next) {
        await resizePanel(false);
        flushSync(() => setExpanded(false));
      }
      setError('');
    } catch { setError('Não foi possível atualizar o painel. Tente novamente.'); }
    finally {
      animation?.cancel();
      busy.current = false;
    }
  }
  useEffect(() => {
    function onKey(e: KeyboardEvent) { if (e.key === 'Escape' && expanded) void toggle(false); }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [expanded]);

  return <main ref={shellRef} className="shell">
    <header className="widget glass">
      <span className="grip" title="Arrastar janela" aria-hidden="true" onPointerDown={e => {
        if (e.button === 0) void dragWindow().catch(() => setError('Não foi possível mover a janela.'));
      }}>⠿</span>
      <button ref={toggleRef} className="widget-toggle" aria-expanded={expanded} aria-controls="panel"
        onClick={() => void toggle(!expanded)}>
        <span className="mark">S</span><span className="brand">SYNC<small>SYSTEM</small></span>
        <span className={'status-dot' + (focus ? ' focusing' : '')} /><span className="status">{focus ? 'Em foco' : 'Seu espaço, em calma'}</span>
        <span className="chevron">{expanded ? '−' : '+'}</span>
      </button>
    </header>
    {expanded && <section ref={panelRef} id="panel" className="panel glass" aria-label="Painel Sync System">
      <div className="eyebrow">SEU PEQUENO CENTRO</div>
      <h1>Menos ruído.<br /><span>Mais presença.</span></h1>
      <p className="intro">Um espaço para organizar o agora.</p>
      <section className="card focus-card"><div><span className="card-icon">◉</span><h2>Modo foco</h2><p>{focus ? 'Um passo de cada vez.' : 'Escolha o que merece sua atenção.'}</p></div>
        <button className="switch" role="switch" aria-checked={focus} aria-label="Modo foco" onClick={() => setFocus(!focus)}><span /></button>
      </section>
      <Notes />
      <footer><span><i />Tudo acontece aqui</span><button onClick={() => void toggle(false)}>Recolher <kbd>Esc</kbd></button>
        {native && <button onClick={() => void closeWindow().catch(() => setError('Não foi possível sair.'))}>Sair</button>}</footer>
    </section>}
    {error && <p className="error" role="alert">{error}</p>}
    {!native && <aside className="preview-label">SYNC SYSTEM / PRÉVIA DO WIDGET NO DESKTOP</aside>}
  </main>;
}