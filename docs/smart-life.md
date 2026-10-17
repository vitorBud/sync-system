# Conectar a tomada Smart Life ao Sync System

O LED é alimentado por uma tomada inteligente cadastrada no Smart Life do iPhone. O conector controla apenas ligar/desligar a tomada; não presume suporte a brilho ou cores.

## Estado da implementação

O conector Tuya Cloud e o formulário estão implementados. Ainda não houve teste com a tomada real, porque a conta Smart Life ainda precisa ser vinculada a um projeto Tuya e configurada pelo usuário.

O caminho adotado nesta etapa é a API oficial Tuya Cloud. A marca/modelo é útil para diagnóstico, mas o conector consulta as funções expostas pela API em vez de presumir um modelo ou um canal fixo.

## 1. Criar o projeto

Abra https://platform.tuya.com e crie sua conta Tuya Developer. A conta do portal é uma configuração separada do app Smart Life.

No portal, procure Cloud > Cloud Project/Development e crie um projeto com método Smart Home. A nomenclatura pode variar na interface do portal.

Escolha o data center que atende a conta Smart Life conforme a orientação do portal. Não selecione a região apenas por proximidade geográfica.

Confira as condições atuais do IoT Core e das APIs necessárias. A Tuya oferece modalidades de teste e planos com limites/validade. Não é possível garantir acesso gratuito permanente; o conector não contrata serviços.

## 2. Vincular o Smart Life

No projeto, entre em Devices > Link App Account/Link Tuya App Account > Add App Account.

Escolha autorização de conta Tuya/Smart Life. Use o Smart Life no iPhone para escanear o QR code mostrado pelo portal e autorize a vinculação da sua conta.

Confirme que a tomada aparece em All Devices no projeto. Se não aparecer, confira o data center e a conta vinculada.

## 3. Preencher no Sync System

Na aba Dispositivos, em LED do quarto, clique em Conectar tomada.

Preencha:
- Data center do projeto.
- Client ID/Access ID.
- Client Secret/Access Secret.
- Device ID da tomada que alimenta o LED.

Se houver uma única função booleana de interruptor, ela será escolhida automaticamente. Se houver múltiplos canais, abra Tomada com mais de um canal e informe o código de interruptor exibido nas funções da tomada no portal Tuya.

Clique em Salvar e conectar à Tuya. Esse passo autentica na Tuya e consulta as funções e o estado; ele não liga nem desliga a tomada.

As chaves são persistidas via armazenamento nativo de credenciais do sistema: Windows Credential Manager no Windows e Keychain no macOS. A versão macOS ainda não foi validada. A configuração não é gravada no localStorage, nos fontes ou no Git. O campo Secret é limpo ao salvar/cancelar e nunca é devolvido pelo backend.

## 4. Testar os controles

Confirme que o dispositivo detectado é a tomada do LED. Use Ligar ou Desligar e observe a luz.

O painel diferencia:
- Não conectada: não há configuração salva.
- Não verificado: existe configuração, mas não há consulta válida recente.
- Offline: a API reportou a tomada offline.
- Ligada/Desligada: estado do relé reportado pela Tuya.
- Comando aceito sem mudança confirmada: a API aceitou o envio, mas a leitura ainda não confirmou o valor solicitado.

O estado reportado pela nuvem pode ter atraso. Não há polling contínuo: consultas ocorrem ao abrir o cartão conectado, atualizar ou enviar uma ação. Falhas removem o estado apresentado, evitando mostrar um valor anterior como atual.

Remover conexão deste computador apaga somente as credenciais locais após confirmação na interface. Não remove a tomada do Smart Life nem revoga o vínculo no portal Tuya.

## APIs utilizadas

- GET /v1.0/token?grant_type=1
- GET /v1.0/iot-03/devices/{device_id}
- GET /v1.0/iot-03/devices/{device_id}/functions
- GET /v1.0/iot-03/devices/{device_id}/status
- POST /v1.0/iot-03/devices/{device_id}/commands

Os destinos HTTPS são limitados aos data centers conhecidos da Tuya. O ID do dispositivo é validado antes de montar o caminho. A assinatura é HMAC-SHA256; o segredo e o token não são incluídos nas mensagens de erro do frontend.

## Verificação de desenvolvimento

npm run build
cargo test --manifest-path src-tauri/Cargo.toml --lib

Os testes usam dados fictícios para conferir assinatura, restrição dos destinos, seleção de canal, estado desconhecido e tratamento de erros. Não acessam uma conta Tuya nem acionam equipamentos.

## Referências oficiais

- Projeto, chaves e vínculo do app: https://developer.tuya.com/en/docs/developer/apply-cloud-api-key?id=Kff30z8sv62ah
- Vínculo Smart Life: https://developer.tuya.com/en/docs/iot/link-devices?id=Ka471nu1sfmkl
- Condições dos serviços: https://developer.tuya.com/en/docs/iot/membership-service?id=K9m8k45jwvg9j
- Assinatura para autorização cloud: https://developer.tuya.com/en/docs/iot/new-singnature?id=Kbw0q34cs2e5g
- Funções do dispositivo: https://developer.tuya.com/en/docs/cloud/3ac29198c9?id=Kag2ybepz3arq
- Comandos: https://developer.tuya.com/en/docs/cloud/e2512fb901?id=Kag2yag3tiqn5
## Conectividade informada pelo usuário

A tomada utiliza Bluetooth na configuração inicial pelo Smart Life e, depois de cadastrada, é controlada por Wi-Fi. O Sync System utilizará a tomada já configurada; esta etapa não inclui refazer o pareamento nem implementar Bluetooth no computador.

O fato de o controle funcionar por Wi-Fi não confirma acesso direto por uma API LAN sem credenciais. O conector implementado usa Tuya Cloud e depende de internet e autorização do projeto. Controle LAN continua sendo uma alternativa a avaliar mediante identificação do dispositivo e das credenciais/protocolo locais.