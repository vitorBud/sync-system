# Núcleo de dispositivos e ações

## Organização

- src/core/devices.ts: contrato de dispositivos, capacidades, ações e configurações.
- src/core/controlCenter.ts: coordenação independente de React/Tauri; evita ações concorrentes e exige capacidade/confirmação antes de encaminhar.
- src/platform/controlTransport.ts: transporte nativo ou prévia explícita do navegador.
- src/features/controls: apresentação e formulários; não executa comandos de fabricante diretamente.
- src-tauri/src/core: catálogo e roteamento validados no backend.
- src-tauri/src/core/settings.rs: configuração local com validação e substituição atômica.
- src-tauri/src/integrations: energia, aplicativos e Sunshine.

## Destinos desta etapa

local_pc representa somente o computador onde esta instalação está executando.
gamer_wake é um destino de despertar por MAC, separado do computador local.
switch_streaming representa o serviço Sunshine local, não o estado online/offline do console.
ps5 representa o acesso ao aplicativo PS Remote Play, não o estado de energia do PS5.

Avell e MacBook não são anunciados como presentes antes de serem instalados e pareados. A descoberta e execução entre computadores ficam para a etapa combinada após consolidar o funcionamento no gamer.

## Execução

A interface encaminha um deviceId e um actionId fechados por enum. O backend valida a combinação, as configurações e a confirmação de desligamento. Não existe execução genérica de shell nem endpoint de comandos exposto na rede.

As verificações acontecem ao abrir a área de dispositivos e ao atualizar ou concluir uma ação. Não existe polling permanente. Uma verificação que falha remove o catálogo anterior da interface; horário da última consulta fica explícito.

O Sunshine é verificado no endpoint HTTP público serverinfo em loopback. Só uma resposta XML válida com estado reconhecido é considerada disponível. Uma sessão ocupada não identifica o cliente como Nintendo Switch. Abrir Sunshine encaminha ao painel HTTPS local; autenticação e eventuais avisos de certificado são tratados pelo usuário no navegador.

Wake-on-LAN lê o destino salvo; o envio não prova que a máquina ligou. Desligamento continua restrito ao computador local, com confirmação modal e sem fechamento forçado de aplicativos.

## Configurações

controls.json fica no diretório de configuração desta instalação. Salvar valida o conteúdo antes de substituir o arquivo. Erros de leitura não sobrescrevem configurações existentes.

A configuração antiga de Wake-on-LAN no localStorage é oferecida como preenchimento inicial do formulário quando não existe destino nativo salvo. Ela só é migrada ao clicar em Salvar configuração.

## LED adiado

Tuya está fora da compilação padrão e fora da interface principal. O código anterior foi preservado em src-tauri/src/tuya.rs e src/features/lighting para retomada.

O recurso Rust opcional lighting-tuya permite compilar/testar esse conector separadamente. Não o habilitar no uso normal nesta etapa. Nenhuma credencial Tuya é lida e nenhuma consulta à nuvem Tuya é feita no fluxo padrão.

## Validação

- npm test: coordenação, confirmação, concorrência, erro e roteamento na interface.
- npm run build: TypeScript e compilação frontend.
- cargo test --manifest-path src-tauri/Cargo.toml --lib: núcleo nativo, energia, configurações e leitura Sunshine.
- cargo test --manifest-path src-tauri/Cargo.toml --lib --features lighting-tuya: inclui o código adiado do LED.

Os testes não desligam equipamentos nem usam destinos reais de Wake-on-LAN.