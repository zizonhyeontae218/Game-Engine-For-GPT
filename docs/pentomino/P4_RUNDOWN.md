# P4 Rundown — Native Bridge & Gameplay Integration

TL;DR: 두 전투자의 최소 턴 전투를 실제 P2/P3 공개 API와 새 Rust/C Native
Bridge에 연결했다. 입력 → 결정 Tick → 적용 Tick → CameraFrame → 기존 CPU
RGBA → 독립 저장/복원 → 실제 규칙 플러그인 제거/재설치 → 종료가 실행된다.
FlatLand 게임 실행을 대체 구현이라고 부르지 않았고 Core/기존 ABI1은 수정하지 않았다.

Status: **PARTIAL**. 구현·내부 검증은 PASS. 독립 새 세션/source-free Consumer
인증은 UNVERIFIED이며 내부 공개-API 예제나 packaged selfcheck로 대체하지 않는다.
[PR #5](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/pull/5) 생성 완료.
구현 commit `51a9a17`의 [원격 acceptance CI](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/38414171986)
PASS. SDK selfcheck와 실제 source-free C/ctypes 실행도 PASS.
외부 인증 이외의 요청된 구현 산출물은 준비됐다.

Checklist:

- [x] Native Bridge 계약: [P4_CONTRACT.md](P4_CONTRACT.md), 버전 구분 Rust/C API.
- [x] 입력 → 고정 두 Core Tick → Gameplay → P3 CameraFrame → 실제 CPU RGBA 전달.
- [x] JSON 설정의 최소 두 전투자 턴 전투, HP·공격·턴·승패.
- [x] Core save2/Presentation save1 분리 및 전투·카메라 전환 중 결정론적 복원.
- [x] 오류 원자성, 위조된 완료 경계·지원 범위 밖 저장 거부, 세션 종료 핸들 무효화.
- [x] 실제 TurnCombat CorePlugin 제거/재설치와 carrier 권위·Scene/Entity 보존.
- [x] Camera/View 교체 시 Core save 완전 불변; 실제 카메라 변경 픽셀 검증.
- [x] 동일 Seed/Input, Headless/Linked 상태 일치 및 공개 API 예제·실제 C ABI Consumer.
- [x] P0–P3 및 기존 다섯 게임 회귀, fmt/Clippy/workspace 테스트.
- [x] P5 Handoff, 재현 스크립트와 검증 증거.
- [ ] 독립 새 세션의 Consumer 인증. 공개 API 최소 예제 실행은 내부 재현 근거이며
  외부 소비자의 독립 시험 결과는 아직 제공되지 않았다.

Validation:

- `cargo fmt --all --check`: PASS.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --locked --workspace`: **169 PASS, 0 FAIL**. Gameplay5,
  Native Rust12 + FFI2 포함. [로그](evidence/p4/test-workspace.log).
- P2/P3/P4 dependency/preservation checks, P3 public/fifth 실제 렌더 예제, P4
  public 예제, CLI no-default-features Clippy/tests, Python tooling/unit checks:
  PASS. [명령·결과](evidence/p4/checks.json).
- Basement, Pac-Man, Signal Yard, Harbor, Nuvema replay: 모두 PASS. 실제
  deterministic SHA256/golden RGBA/PNG/save-reload 결과를 개별 legacy 로그에 보존.
- 소스 없는 `/workspace/ge4g-p4-public-sdk`에서 공개 C 헤더 compile/link와
  cdylib ctypes Consumer 실행: PASS. 소비자 런타임은 engine source를 읽지 않는다.
  SDK ZIP SHA256 `7d7651d7a762af7fa6e97f30745b5a160575d7ecad212997b29b68532d793864`.
  [manifest](evidence/p4/sdk-manifest.json), [source-free 결과](evidence/p4/sourcefree-result.json).
- C header compile/link smoke와 실제 cdylib ctypes Consumer: PASS.
  [Consumer 로그](evidence/p4/ffi-consumer.log), [C 로그](evidence/p4/c-smoke.log).
- P4 public 예제: 5회 공격, Core Tick10, HP `[6,0]`, winner0. 실제 PNG
  [이전](evidence/p4/before.png) / [이후](evidence/p4/after.png).
  네이티브 전달 최종 RGBA SHA256:
  `6ef9dc3aaff45923a9a346d1fd7e71e0dbdef9efc0c1541d7654751f61f9f0f4`.
- Independent auditor: 네 결함을 재현·수정·회귀 시험으로 닫았고 최종 검토 범위에
  남은 HIGH/MEDIUM 없음. Narrow14/14 재실행 PASS. [기록](evidence/p4/audit.json).
- Baseline main `8622034`, [acceptance CI](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37926779602)
  PASS. 현재 PR/CI는 [최종 근거](evidence/p4/verification.json)에서 확인한다.
- Rust1.99.0, x86_64-unknown-linux-gnu 개발 환경. 초기 Rust/ALSA dev 부재를
  로컬 도구 설치로 해결했고 proxy/TLS를 유지했다. 테스트 조건을 약화하지 않았다.
  테스트 lint 수정과 증거 축적 중 초기 FILETREE stale 실패도 별도 로그로 보존했다.

Compatibility:

- 기존 P0–P3와 legacy source/test/API hash 보존 검사 PASS. 기존 save1/save2,
  FlatLand 저장, ABI1, game schema/package, Flutter UI·앱 ID·인증서·버전 유지.
- 새 crate2개와 `ge4g_p4_v1_*` C symbol, config/observation/save envelope v1을
  추가했다. 기존 의존성 버전은 그대로이고 Cargo.lock에는 새 crate 항목만 추가했다.
- 실제 P2 remove 의미를 바꾸지 않았다. 제거 대상은 규칙 owner이며 carrier는
  영속 상태를 유지한다. 전체 Core hash는 제거 시 달라지는 것이 정상이다.
- main/release/0.2를 덮어쓰거나 PR을 병합하거나 정식 릴리즈를 게시하지 않았다.

Known Limitations:

- 독립 새 세션 Consumer 인증, Android/Windows 실행물·실기기·서명/릴리즈는
  UNVERIFIED. Linux-hosted native SDK는 개발 근거이며 새 Linux client 지원이 아니다.
- 최소 고정 피해 두 전투자만 구현했다. 완성형 턴 전투, 일반 rules migration,
  동적 executable plugin loader, realtime/open world/platformer/Forge 없음.
- C 세션은 생성 Thread에 귀속된다. RGBA JSON 배열과 전체 후보 호스트 복원은
  정확성 우선 구현이며 대규모 부하/대역폭·성능 개선은 측정 후 진행해야 한다.
- 실제 렌더러의 지원된 orthographic subset만 사용한다. Full3D GPU, 일반 회전/
  perspective renderer, 전용 완성형 전투 UI·플랫폼별 float byte 동일성은 미검증.
- trusted native callback 외부 부작용, 악성 native pointer/OOM/process failure
  recovery, 저장 진본 인증을 보장하지 않는다. 기록 일관성과 원자성은 검증했다.

Next: [P4_HANDOFF.md](P4_HANDOFF.md)의 P5 Turn Combat 계약·규칙 확장을 진행하고,
독립 source-free Consumer 시험 결과를 수집한다. 이후 Playable Alpha 플랫폼
연결을 실제로 검증한다. Tiny Core에 전투 규칙을 넣거나 P0–P3를 반복하지 않는다.
