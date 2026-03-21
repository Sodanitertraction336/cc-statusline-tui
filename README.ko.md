# claude-statusline-config

> Claude Code 상태표시줄, 너무 밋밋하지 않나요? 한번 꾸며봅시다.

[English](README.md) | [中文](README.zh.md) | [日本語](README.ja.md) | [Español](README.es.md) | [Português](README.pt.md) | [Русский](README.ru.md)

![statusline preview](preview.png)

명령어 하나. 7개 언어. 후회 제로.

## 설치

취향대로 골라주세요:

```bash
# npm — 정석
npx claude-statusline-config

# Homebrew — 있어보이는 선택
brew tap LokiQ0713/claude-statusline-config && brew install claude-statusline-config

# Cargo — Rustacean의 품격
cargo install claude-statusline-config
```

## 뭐가 되는 건데?

TUI 마법사가 뜹니다. 이것저것 고르면 됩니다. 멋져집니다. 끝.

```
1/4 세그먼트      → 뭘 보여줄지
2/4 설정          → 어떻게 보일지 (실시간 미리보기, 즉각 반영)
3/4 순서 정하기    → 어디에 배치할지
4/4 확인          → 적용!
```

커스터마이즈 귀찮으시다고요? "기본값 사용" 선택하고 Enter 두 번이면 끝. 아무도 뭐라 안 합니다.

## 세그먼트

| 세그먼트 | 이렇게 보여요 | 하는 일 | 바꿀 수 있는 것 |
|---------|-------------|--------|---------------|
| Model | `🔥 Opus4.6` | 지금 쓰고 있는 모델 표시 | 아이콘 (🔥🦊🐢🐰), 색상 |
| Cost | `$0.42` | 지갑이 얼마나 아픈지 실시간 확인 | 색상 |
| Usage | `██░░ 25% 1h43m` | 5시간 사용량 제한: 바 + % + 리셋 카운트다운 | 바 스타일/문자/길이, 색상, 갱신주기 |
| Path | `~/project` | 현재 위치 | 색상, 최대 길이 |
| Git | `main* ↑2↓1` | 브랜치 + 변경사항 + ahead/behind | 표시 항목, 색상 |
| Context | `▓▓▓▓░░░ 60% 600K/1M` | 컨텍스트 윈도우 얼마나 먹었는지 | 바 스타일/문자/길이, 색상 |
| Crypto | `BTC:$73748` | 코딩하다가 코인 시세도 확인. 왜냐고요? 그냥요. | 코인 (BTC/ETH/BNB/SOL), 색상, 갱신주기 |

## 색상 스타일

| 스타일 | 느낌 |
|-------|------|
| **Ultrathink Rainbow** | 글자마다 색이 다름. 네, 반짝반짝합니다. |
| **Ultrathink Gradient** | 바 전체에 부드러운 무지개 그라데이션. 감동 그 자체. |
| **Traffic Light** | 초록 (≤30%) → 노랑 (≤60%) → 빨강 (>60%). 컨텍스트 윈도우가 막히고 있어요. |
| Cyan / Green / Blue / Yellow | 무난한 선택 |
| Magenta / Red / Orange / Pink / Purple / White | 개성 넘치는 선택 |

## 지원 언어

English, 中文, 日本語, 한국어, Español, Português, Русский.

첫 실행 시 선택. 상태표시줄이 당신의 언어로 말합니다.

## 작동 원리

1. 마법사가 설정을 `~/.claude/statusline/config.json`에 저장
2. 바이너리가 `~/.claude/statusline/bin/`에 복사됨
3. `~/.claude/settings.json`이 자동으로 업데이트됨
4. Claude Code 재시작. 새 상태표시줄 감상. 동료한테 자랑.

다시 실행하면 기존 설정이 기본값으로 불러와집니다. 기존 설정 안 날아갑니다. 약속.

## 요구사항

- Claude Code 설치됨 (`~/.claude/`가 존재해야 함)
- Node.js ≥ 18 (`npx` 설치 시에만 필요 — 실행 시에는 불필요)

## 기여하기

버그 발견? 기능 제안? [이슈를 열어주세요](https://github.com/LokiQ0713/claude-statusline-config/issues). PR 환영합니다.

## License

MIT
