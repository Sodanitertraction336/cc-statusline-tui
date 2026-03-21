# claude-statusline-config

> A barra de status do seu Claude Code tá sem graça. Bora dar um trato nela.

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Español](README.es.md) | [Русский](README.ru.md)

![statusline preview](preview.png)

Um comando. Sete idiomas. Zero arrependimentos.

## Instalação

Escolhe teu método:

```bash
# npm — o clássico
npx claude-statusline-config

# Homebrew — pra quem tem bom gosto
brew tap LokiQ0713/claude-statusline-config && brew install claude-statusline-config

# Cargo — pros Rustáceos de plantão
cargo install claude-statusline-config
```

## O que acontece?

Um wizard TUI aparece. Você escolhe umas coisas. Fica bonito. É isso.

```
1/4 Segmentos     → O que mostrar
2/4 Configurar    → Como vai ficar (preview ao vivo, feedback instantâneo)
3/4 Reordenar     → Onde cada coisa fica
4/4 Confirmar     → Manda ver!
```

Tá com preguiça de customizar? Escolhe "Usar padrões" e aperta Enter duas vezes. A gente não julga.

## Segmentos

| Segmento | Aparência | O que faz | Dá pra ajustar |
|----------|----------|-----------|----------------|
| Model | `🔥 Opus4.6` | Mostra qual cérebro você tá usando | Ícone (🔥🦊🐢🐰), cor |
| Cost | `$0.42` | O nível de sofrimento da sua carteira | Cor |
| Usage | `██░░ 25% 1h43m` | Limite de 5h: barra + % + contagem regressiva | Estilo/caractere/tamanho da barra, cor, atualização |
| Path | `~/project` | Onde você tá | Cor, tamanho máximo |
| Git | `main* ↑2↓1` | Branch + alterações + ahead/behind | Componentes, cor |
| Context | `▓▓▓▓░░░ 60% 600K/1M` | Quanto da janela de contexto você já comeu | Estilo/caractere/tamanho da barra, cor |
| Crypto | `BTC:$73748` | Porque por que não checar preços enquanto programa? | Moedas (BTC/ETH/BNB/SOL), cor, atualização |

## Estilos de cor

| Estilo | A vibe |
|--------|--------|
| **Ultrathink Rainbow** | Cada caractere é de uma cor diferente. Sim, ele brilha. |
| **Ultrathink Gradient** | Arco-íris suave ao longo da barra. De cair o queixo. |
| **Traffic Light** | Verde (≤30%) → Amarelo (≤60%) → Vermelho (>60%). Sua janela de contexto virou um congestionamento. |
| Cyan / Green / Blue / Yellow | As escolhas sensatas |
| Magenta / Red / Orange / Pink / Purple / White | As escolhas expressivas |

## Idiomas

English, 中文, 日本語, 한국어, Español, Português, Русский.

Selecionado na primeira execução. Sua barra de status fala a sua língua.

## Como funciona

1. O wizard salva a configuração em `~/.claude/statusline/config.json`
2. O binário vai pra `~/.claude/statusline/bin/`
3. `~/.claude/settings.json` é atualizado automaticamente
4. Reinicie o Claude Code. Admire sua nova barra de status. Conte pros colegas.

Rodar de novo carrega sua configuração existente como padrão. Não destrói nada. Promessa.

## Requisitos

- Claude Code instalado (`~/.claude/` precisa existir)
- Node.js ≥ 18 (só pra instalar via `npx` — não precisa no runtime)

## Contribuindo

Achou um bug? Quer uma feature? [Abra uma issue](https://github.com/LokiQ0713/claude-statusline-config/issues). PRs são bem-vindos.

## License

MIT
