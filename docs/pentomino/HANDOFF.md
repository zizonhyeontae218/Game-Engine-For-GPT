# GE4G 0.3 Pentomino — 다음 작업자 시작점

확인일: 2026-10-08. 이번 작업은 PentaWorks 설치/인수인계 설정이다.
0.3 엔진 구현, 버전 변경, 제품 재배포는 아직 하지 않았다.

## 우선순위와 시작 순서

최신 사용자 지시 → 기존 AGENTS.md의 호환성/제품 규칙 → 이 인수인계와
BRIEF.md의 0.3 목표 순으로 읽는다. BRIEF는 첨부 PentaWorks의 설계 목표이며,
독립 원본 3페이지 문서는 제공되지 않았다. 제안 API는 현재 API가 아니다.

퀵스타트의 첫 작업 지시문을 받으면 다시 시작 허가를 묻지 않고:

1. git status/HEAD/remotes와 실제 도구를 확인한다. 사용자 변경은 보존한다.
2. pentomino_scout를 읽기 전용으로 호출한다. BOOTSTRAP_PACKETS.md의 P0-S를
   사용하여 실제 모듈·API·테스트·경계 위험을 조사한다.
3. .agent/PLANS.md에 맞는 active ExecPlan을 작성하고 Tiny Core → View/Format →
   Gameplay 마일스톤을 정한다. 파일 소유권은 TASK_PACKET.md로 배정한다.
4. pentomino_architect의 최소 공개 API 초안 → pentomino_auditor 독립 검토.
   부모가 검토 결과를 통합한 뒤 P1의 작은 구현 단위로 진행한다.
5. 기능/테스트 증거를 GATES.md 및 RUNDOWN.md에 기록한다. 증거 없는 항목은
   UNVERIFIED. P0 조사/문서 결과만으로 P1 구현 완료를 주장하지 않는다.

이전 Soul.md의 “즉시 0.3을 열지 말 것”은 당시 인수인계 작업의 범위였다.
현재 사용자는 다음 작업자가 명시적인 퀵스타트 지시로 0.3을 시작하도록
설정을 요청했다. 설정 완료 자체가 제품 버전/지원 플랫폼 변경은 아니다.

## 확인한 0.2 출발점

- 저장소: https://github.com/zizonhyeontae218/Game-Engine-For-GPT
- 설치 전 main/태그 대상: ed3301c7af12ac36dd0b5074d79693d203aeeb28.
  실제 릴리스 빌드 소스: cdb7eb1d336dee95ea2b1e4d82ddc93d478e4098.
- 배포 제품은 FlatLand 0.2.0 FINAL. Rust 0.2.0, Flutter 0.2.0+8.
  0.2 최종화/rc6/재서명/재배포를 반복하지 않는다.
- Cargo Rust 2024 workspace 7개:
  ge4g-core, ge4g-project, ge4g-runtime, ge4g-render2d,
  ge4g-platform, ge4g-client, ge4g-cli. 클라이언트는 client/ Flutter/Dart.
- ge4g-core는 현재 타입/수학/입력/상태/이벤트 경계다. 새 Tiny Core와 같은
  것으로 단정하지 않는다. runtime/render2d/project 의존성을 실제 조사한다.
- 기존 경계: docs/ARCHITECTURE.md, FILETREE.md. authoritative Rust simulation,
  fixed timestep, seeded RNG/replay, schema1/ABI1 및 schema2 content-bound save,
  exact-once 결과 적용을 유지한다. Lua5.4와 실제 오디오 어댑터가 이미 있다.
- 카메라는 presentation-only. 충돌/위치/plane/RNG/전투 결과와 분리한다.
  0.2 건물 facing은 south만 지원하며 다른 방향은 reject한다.
- game_id의 설정/저장/순서와 ZIP bytes digest의 역할을 보존한다.
  data-only 게임 import에 임의 executable plugin을 넣는 것은 승인된 계약이 아니다.
  plugin 설치/신뢰/호스트 실행 경계는 P0 설계에서 명시한다.
- Android ID dev.ge4g.ge4g_client 및
  client/android/signing-certificate.sha256의 인증서를 유지한다.
  키/암호는 출력하거나 저장소에 넣지 않는다. 새 키를 생성하지 않는다.
- 현재 지원 클라이언트는 Android/Windows. 0.3 시작 시 지원 matrix를
  명시적으로 재검토하되 iOS/macOS/Linux/Arch CI를 자동 활성화하지 않는다.
  Linux Rust/headless 검사와 Android cross-build는 개발 인프라다.
- Forge/스토리/에셋 생성은 0.4 Mr. Smith. marketplace/cloud/network updater는
  이번 P0 목표에 포함하지 않는다. 공개 샘플은 CC0 바람항 공방.

## 알려진 미해결 Windows 실패

태그 clients run 37486665662의 Android는 성공, Windows job 112348341230은
Flutter test에서 실패했고 이후 build/embedded launch는 skipped였다.
2026-10-08 GitHub jobs readback에서도 이 상태를 확인했다.
이전 빌드 소스의 성공 증거와 혼동하지 않는다.

- client/test/final_library_test.dart:
  normal deletion preserves user data, reinstall reuses it; full deletion removes archives/settings.
- 163행 expect(installed.digest, game.digest). 첫 설치/재설치의 fixture.package()
  호출이 새 ZIP bytes를 만들며 timestamp 차이가 원인이라는 가설이 있다.
- 다음 관련 검증에서 client/test/library_test.dart의 package()와 archive
  timestamp를 확인하고, 동일 bytes 재사용 또는 고정 metadata로 의도를 검증한다.
  시간 경계 재현과 Windows 회귀가 필요하다. 현재 원인/해결은 UNVERIFIED.
- 삭제/데이터 보존 assertions와 엔진 digest/save validation을 약화하지 않는다.
  이 실패 때문에 렌더러/전투를 재설계하지 않는다.

## 검증 경로

설정 검증:

    python3 scripts/validate_pentaworks.py
    python3 -m unittest discover -s tests -v
    python3 scripts/release_consistency.py --self-test
    python3 scripts/filetree.py lint

위 명령은 엔진 플러그인 기능/실기기/소비자 격리 성공의 증거가 아니다.
기능 변경 시 narrow tests → docs/TESTPLAN.md와 .github/workflows/{ci,client}.yml.
Rust headless 검사 후 기본/feature 빌드 출력이 덮어써지는 점도 주의한다.
작업자 실제 spawn과 새 클라이언트의 권한 적용은 실행 시 별도로 확인한다.

consumer는 공개 문서/배포물만 별도 source-free workspace와 새 세션에서
받는다. consumer/ALLOWLIST.example.txt는 아직 실제 공개 목록이 아니며,
stage_consumer_bundle.py만으로 OS 수준 격리가 이루어지지 않는다.

## 근거와 복구 링크

- [Drive Soul.md](https://drive.google.com/file/d/1uolemYGUhF3RgQ9OxWtKa-GJrjnsWfCf/view)
  (2026-10-08 작성; 사용자 계정의 접근 권한 필요).
- [Drive 0.2.0](https://drive.google.com/drive/folders/1iJPNZLzYvo3CRjxqfKQAOmqC0vNIjM_f):
  Soul.md, README-ko.txt와 배포 파일 목록을 읽었다. 기존 파일/공유 상태를 유지한다.
- [v0.2.0 릴리스](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/tag/v0.2.0),
  docs/releases/flatland-final-verification.json, flatland-final-drive.json.
- [후속 실패](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37486665662).
- [Codex multi-agent 공식 설정](https://developers.openai.com/codex/multi-agent):
  project .codex/agents/*.toml의 name/description/developer_instructions 및
  [agents] enabled/max_concurrent_threads_per_session 형식을 확인했다.
  설정은 모델을 고정하지 않고 부모 모델/권한을 상속한다. scout/auditor/consumer는
  read-only 선언. 새 세션에서 프로젝트 설정 trust/스킬 discovery를 확인한다.
- 첨부 PentaWorks_v1.0_Codex_Agent_Set.zip과 QUICKSTART.ko.md.
  전체 Soul의 로컬 경로/과거 업로드 명령을 재실행하지 않는다.
