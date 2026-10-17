# Sync System

Widget discreto para Windows, com painel sob demanda, visual translúcido e animações suaves. Protótipo inicial em React, TypeScript e Tauri 2/Rust. macOS será validado em uma etapa posterior.

## Executar

Requisitos para desktop Windows: Node.js, Rust stable MSVC, Visual Studio Build Tools com C++ e Windows SDK, e WebView2. Consulte os [pré-requisitos oficiais do Tauri](https://v2.tauri.app/start/prerequisites/).

```powershell
npm ci
npm run desktop
```

Para visualizar somente a interface: execute `npm run dev` e abra http://127.0.0.1:1420. Não execute esse servidor em paralelo com `npm run desktop`, que já inicia o frontend.

Se o Cargo não for encontrado após a instalação, reabra o VS Code para atualizar o PATH.

## Funcionalidades

- Widget flutuante com abertura e recolhimento animados; Escape recolhe.
- Janela ajustada à altura do painel, sem rolagem externa.
- Alça para arrastar a janela nativa.
- Notas salvas apenas neste dispositivo, no localStorage.
- Modo foco visual, sem bloquear notificações ou iniciar temporizador.
- Botão Sair para encerrar a janela desktop.

## Arquitetura

- `src/features`: funcionalidades isoladas, inicialmente notas.
- `src/platform`: ponte com a janela nativa e suporte à prévia no navegador.
- `src/App.tsx`: composição e transições do widget e painel.
- `src-tauri`: host Rust com comando limitado de redimensionamento.

Sem integrações remotas, Alexa, telemetria, polling ou animações contínuas. Dependências, arquivos locais de ambiente e resultados de compilação não são versionados. Os arquivos de lock são versionados para reproduzir as dependências.

## Validação

```powershell
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

O protótipo inicial foi compilado e iniciado no Windows. O consumo em repouso e o comportamento em múltiplos monitores/DPI ainda precisam ser medidos.

## Próximos passos

O vidro é uma aproximação em CSS; não é Liquid Glass nativo nem desfoque garantido do papel de parede. A janela ainda não tem ancoragem ao desktop, bandeja, inicialização automática ou always-on-top. A compilação macOS depende do ambiente Apple e ainda não foi validada.

Antes de distribuir: definir identificador final, ícones e assinatura e habilitar bundle. O identificador interno original permanece para preservar o armazenamento local existente.