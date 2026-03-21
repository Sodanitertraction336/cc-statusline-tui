# claude-statusline-config

> Tu barra de estado de Claude Code es aburrida. Vamos a arreglar eso.

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md) | [Português](README.pt.md) | [Русский](README.ru.md)

![statusline preview](preview.png)

Un comando. Siete idiomas. Cero arrepentimientos.

## Instalación

Elige tu veneno:

```bash
# npm — el clásico de toda la vida
npx claude-statusline-config

# Homebrew — para paladares exigentes
brew tap LokiQ0713/claude-statusline-config && brew install claude-statusline-config

# Cargo — para los Rustáceos
cargo install claude-statusline-config
```

## ¿Qué pasa?

Aparece un asistente TUI. Eliges cosas. Queda bonito. Fin.

```
1/4 Segmentos     → Qué mostrar
2/4 Configurar    → Cómo se ve (vista previa en vivo, feedback instantáneo)
3/4 Reordenar     → Dónde va cada cosa
4/4 Confirmar     → ¡A producción!
```

¿Te da pereza personalizar? Elige "Usar valores por defecto" y pulsa Enter dos veces. Aquí no juzgamos a nadie.

## Segmentos

| Segmento | Se ve así | Qué hace | Puedes ajustar |
|----------|----------|----------|----------------|
| Model | `🔥 Opus4.6` | Muestra qué cerebro estás quemando | Icono (🔥🦊🐢🐰), color |
| Cost | `$0.42` | El nivel de dolor de tu cartera | Color |
| Usage | `██░░ 25% 1h43m` | Límite de 5h: barra + % + cuenta atrás | Estilo/carácter/longitud de barra, color, actualización |
| Path | `~/project` | Dónde estás | Color, longitud máxima |
| Git | `main* ↑2↓1` | Rama + cambios sin commit + ahead/behind | Componentes, color |
| Context | `▓▓▓▓░░░ 60% 600K/1M` | Cuánta ventana de contexto te has comido | Estilo/carácter/longitud de barra, color |
| Crypto | `BTC:$73748` | Porque ¿por qué no mirar precios mientras programas? | Monedas (BTC/ETH/BNB/SOL), color, actualización |

## Estilos de color

| Estilo | La onda |
|--------|---------|
| **Ultrathink Rainbow** | Cada carácter es de un color diferente. Sí, brilla. |
| **Ultrathink Gradient** | Arcoíris suave a lo largo de la barra. Obra de arte. |
| **Traffic Light** | Verde (≤30%) → Amarillo (≤60%) → Rojo (>60%). Tu ventana de contexto es un atasco. |
| Cyan / Green / Blue / Yellow | Las opciones sensatas |
| Magenta / Red / Orange / Pink / Purple / White | Las opciones expresivas |

## Idiomas

English, 中文, 日本語, 한국어, Español, Português, Русский.

Se selecciona en la primera ejecución. Tu barra de estado habla tu idioma.

## Cómo funciona

1. El asistente guarda la configuración en `~/.claude/statusline/config.json`
2. El binario va a `~/.claude/statusline/bin/`
3. `~/.claude/settings.json` se actualiza automáticamente
4. Reinicia Claude Code. Admira tu nueva barra de estado. Cuéntaselo a tus compañeros.

Si lo ejecutas de nuevo, carga tu configuración existente como valores por defecto. No destruye nada. Prometido.

## Requisitos

- Claude Code instalado (que exista `~/.claude/`)
- Node.js ≥ 18 (solo para instalar con `npx` — no se necesita en tiempo de ejecución)

## Contribuir

¿Encontraste un bug? ¿Quieres una feature? [Abre un issue](https://github.com/LokiQ0713/claude-statusline-config/issues). Los PRs son bienvenidos.

## License

MIT
