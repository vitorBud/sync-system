# Sync System

Widget discreto para Windows com painel sob demanda, visual translúcido e animações suaves. React + TypeScript + Tauri 2/Rust. A implementação macOS será validada depois.

## Executar

Pré-requisitos Windows: Node.js 22.18+ (testes usam execução de TypeScript do Node), Rust stable MSVC, Visual Studio Build Tools com C++/Windows SDK e WebView2. Veja os [pré-requisitos do Tauri](https://v2.tauri.app/start/prerequisites/).

~~~powershell
npm ci
npm run desktop
~~~

Para ver apenas a interface: npm run dev e http://127.0.0.1:1420. A prévia do navegador não executa ações de sistema. Não execute esse servidor junto com npm run desktop, que inicia o frontend automaticamente.

Se o Cargo não for encontrado após instalar Rust, reabra o VS Code para atualizar o PATH.

## Nesta etapa

- Widget com abertura/recolhimento suaves; Escape recolhe.
- Bandeja com Mostrar, Ocultar e Sair; fechar a janela oculta.
- Posição restaurada entre sessões e proteção contra múltiplas instâncias.
- Área Dispositivos alimentada por catálogo e capacidades validados no backend.
- Este computador: identificação local e desligamento com confirmação.
- Acordar PC gamer: destino Wake-on-LAN configurável, separado do computador local.
- Streaming para Switch: verificação real do Sunshine local e acesso ao painel para pareamento.
- PlayStation 5: detecção e abertura do PS Remote Play.
- Área Pessoal: notas locais e foco visual.
- LED adiado para o final, desabilitado na compilação padrão.

Ações e estados têm limites explícitos: Sunshine Ligado indica apenas serviço respondendo, não Switch conectado; Remote Play instalado não confirma PS5 online; pacote Wake-on-LAN enviado não confirma PC ligado. Não existe desligamento remoto nesta etapa.

## Configurar a rede

Em Dispositivos > Configurar rede, salve o MAC e o broadcast do destino que deseja despertar e a porta HTTP do Sunshine local (47989 por padrão). As configurações ficam no diretório de configuração deste computador. Dados antigos de Wake-on-LAN são oferecidos no formulário para migração mediante salvamento.

Para ligar o gamer desligado, o comando precisa partir de outro equipamento ativo. BIOS, placa de rede e modo de energia precisam aceitar Wake-on-LAN.

## Arquitetura

Consulte [núcleo de dispositivos e ações](docs/architecture.md).

O frontend mostra o catálogo e solicita ações ao núcleo; os provedores executam integrações específicas. Sem telemetria, polling contínuo ou serviço remoto de comandos. Dependências, credenciais locais e resultados de compilação não são versionados.

## Verificação

~~~powershell
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib
~~~

O shell foi compilado e executado no Windows. Os testes não desligam o computador. O usuário confirmou em 17/10/2026 que controlou o PC gamer pelo Nintendo Switch usando Moonlight com Sunshine. A transmissão real está validada por esse teste manual. O indicador do SYNC foi simplificado para Ligado quando o serviço Sunshine responde e Indisponível quando não há resposta válida; ele não representa uso ativo do Switch. Consumo em repouso, múltiplos monitores/DPI e despertar com o alvo desligado ainda exigem validação no ambiente.

## Ordem de desenvolvimento

1. Consolidar núcleo, controles locais, energia, Sunshine/Switch e PS Remote Play.
2. Instalar no Avell e implementar identidade, pareamento e disponibilidade entre computadores.
3. Adaptar e validar no MacBook.
4. Retomar iluminação Smart Life e decidir o provedor de automação residencial.

O código Tuya já preparado permanece preservado, com [guia de referência](docs/smart-life.md), mas não é requisito para testar o restante. O recurso Rust opcional lighting-tuya permite verificá-lo separadamente.

## Limites do protótipo

O vidro é CSS, sem garantia de desfoque do papel de parede. Ainda não há ancoragem ao desktop, inicialização automática ou always-on-top. Antes de distribuir: definir identificador final, ícones, assinatura e habilitar bundle. O identificador interno original permanece para preservar o armazenamento local.