# Configuração da API CofreSenhaRust

## Localização do arquivo de configuração

O arquivo de configuração está localizado em:

```
%LOCALAPPDATA%\CofreSenhaRust\config.json
```

No Windows, isso geralmente é:

```
C:\Users\[seu_usuario]\AppData\Local\CofreSenhaRust\config.json
```

## Como editar a configuração

1. Abra o explorador de arquivos
2. Cole o caminho acima na barra de endereço
3. Abra o arquivo `config.json` com um editor de texto (Notepad, VS Code, etc.)
4. Edite os valores desejados
5. Salve o arquivo
6. Reinicie o tray app (feche e abra novamente)

## Opções disponíveis

### `api_port`

- **Descrição**: Porta em que a API local escuta
- **Padrão**: `5474`
- **Tipo**: String
- **Exemplo**: `"api_port": "5475"`

### `session_ttl_secs`

- **Descrição**: Timeout de inatividade da sessão em segundos
- **Padrão**: `7200` (2 horas)
- **Tipo**: String
- **Exemplo**: `"session_ttl_secs": "3600"` (1 hora)

### `session_max_ttl_secs`

- **Descrição**: Vida máxima absoluta da sessão em segundos
- **Padrão**: `43200` (12 horas)
- **Tipo**: String
- **Exemplo**: `"session_max_ttl_secs": "86400"` (24 horas)

## Exemplo de arquivo config.json

```json
{
  "api_port": "5474",
  "session_ttl_secs": "7200",
  "session_max_ttl_secs": "43200"
}
```

## Fallback de configuração

Se o arquivo `config.json` não existir ou tiver erros, o tray app usará:

1. Variáveis de ambiente (do arquivo `.env` na raiz do projeto)
2. Valores padrão embutidos no código

## Notas

- Todos os valores devem ser strings válidas
- Os timeouts devem ser números válidos (em segundos)
- A porta deve ser um número entre 1 e 65535
- Após editar, reinicie o tray app para que as mudanças tenham efeito
- Se houver erro no JSON, o arquivo será ignorado e os padrões serão usados
