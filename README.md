# GE4G 0.3.0 — Pentomino

AI 에이전트가 게임을 작성하고 검증하는 결정적 게임 엔진입니다.
**main은 `0.3.0-alpha.1` 개발선**입니다. Pentomino는 GE4G 0.3.0의 버전명이며,
정식 0.3 배포를 뜻하지 않습니다. 최신 안정판은 **FlatLand 0.2.0**입니다.

## 다운로드와 실행

[GitHub Releases](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/tag/v0.2.0)에서
0.2 안정판을 제공합니다. 배포 파일 구성과 공개 상태는 [릴리즈 안내](docs/releases/README.md)를 확인하세요.

- **Android:** 서명된 실행기 APK 설치 → `.ge4g` 게임 가져오기 → 실행.
- **Windows:** 게임·실행기·런타임이 포함된 ZIP 압축 해제 → `ge4g_client.exe` 실행.
- **게임 제작:** [바람항 공방 샘플](examples/flatland_harbor/README.md),
  [저작 빠른 시작](docs/FLATLAND_QUICKSTART.md), [C ABI 예제](examples/c_abi/README.md).
- **에이전트 개발:** [한국어 퀵스타트](QUICKSTART.ko.md), [AGENTS.md](AGENTS.md),
  [현재 상태와 다음 작업](docs/pentomino/STATUS.md).

Actions 아티팩트는 CI 검증용입니다. 서명되지 않은 Android CI APK는 배포용 APK가 아닙니다.
공개 배포는 GitHub Releases에서 받으며 비공개 Google Drive 권한이 필요하지 않아야 합니다.

## 현재 개발 범위

P0–P3를 통합합니다. Tiny Core는 Scene/Entity identity, typed records, inputs,
per-owner history와 save2/discovery를 제공합니다. P3는 독립 View/Format과 다중
Camera, 좌표 변환, 저장 가능한 부드러운 2.5D 전환을 제공합니다.
기존 FlatLand runtime과 P1 scalar/save1은 유지합니다. Gameplay 플러그인과
전체 3D renderer는 아직 구현 범위에 포함되지 않습니다.
[P2 계약](docs/pentomino/P2_CONTRACT.md) · [P3 계약](docs/pentomino/P3_CONTRACT.md) ·
[다음 작업](docs/pentomino/NEXT.md).

| 개발선 | 용도 |
|---|---|
| `main` | GE4G 0.3.0 Pentomino 알파 개발 |
| `release/0.2` | FlatLand 0.2 안정판 유지보수 |
| P0–P3 | 완료된 마일스톤. 새 작업은 main에서 시작 |

Rust workspace는 `0.3.0-alpha.1`, Flutter는 `0.3.0-alpha.1+9`입니다.
Android·Windows가 현재 클라이언트 검증 대상입니다. 기존 앱 ID와 서명 인증서를 유지합니다.

## 소스에서 시작

```sh
git clone https://github.com/zizonhyeontae218/Game-Engine-For-GPT.git
cd Game-Engine-For-GPT
cargo run --locked -p ge4g-cli -- validate examples/basement_demo
cargo run --locked -p ge4g-cli -- test examples/basement_demo
cargo run --locked -p ge4g-cli --no-default-features -- run examples/basement_demo --headless --ticks 60 --json
```

이 CLI와 게임 샘플은 main에 보존된 FlatLand 실행 경로를 사용합니다.
Pentomino 호스트와 연결된 게임 예제로 해석하지 마세요.
0.2만 작업한다면 `git switch release/0.2`로 전환합니다.

[문서 목차](docs/README.md) · [클라이언트 빌드](docs/CLIENT.md) ·
[CLI 계약](docs/CLI_CONTRACT.md) · [검증 기준](docs/TESTPLAN.md) ·
[릴리즈 기록](docs/RELEASE_NOTES.md) · [MIT 라이선스](LICENSE)
