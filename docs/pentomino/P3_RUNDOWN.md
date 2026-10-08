# Rundown — Pentomino P3 Camera / internal candidate

**TL;DR:** 같은 Core 게임 상태를 유지하면서 Camera/View를 교체하는 독립 계층을
완성했다. 기본 smooth2.5D 전환과 기존 다섯 번째 데모의 실제 화면·저장 연속성을
내부 검증했다. **EXTERNAL VALIDATION PENDING**.

## Status

| 목표 | 상태 | 근거 |
|---|---|---|
| Tiny Core 독립성 | PASS | source/dependency 감사,31파일 보존, Core 변경0 |
| View/Camera 교체·독립 lifecycle | PASS | 독립 Core bytes/identity/RNG/event 보존 테스트 |
| Multiple Cameras / viewport | PASS | 독립 active sets/local state; 최대32Camera 구조 |
| Coordinate model / projection | PASS | explicit spaces, depth roundtrip, actual perspective |
| Classic2D / TopDown / Side / Vertical | PASS | actual extractors + distinct follow-axis policies |
| Default smooth2.5D / instant / retarget | PASS | tick state, quaternion/View/lens interpolation 및4096tick edge |
| Canonical save/restore continuation | PASS | exact bytes/output continuation; atomic malformed rejection |
| 기존 fifth fixture / legacyfive | PASS | Nuvema real pixels/82entities; five replay/frame/save regressions |
| Structured discovery | PASS | public consumer capability/requirements selection tests |
| Mixed2D+3D representation | PARTIAL | native3D math/Model/Background/Billboard; full GPU renderer 미구현 |
| Performance | PASS (측정 범위) |1/16/32camera,256visible,4096real source selection; uplift 주장 없음 |
| External GPT Work | UNVERIFIED | 별도 세션 보고서 대기 |

## Architecture

`CoreHost::select → ReadFrame → FormatPlugin/ViewHost → CameraFrame/RenderFrame`.
Native crate는 Core만 의존하며 mutable Core/World를 받지 않는다. Camera-local
transition/follow/shake/active state는 별도 presentation save1에 저장한다.
World/View/Camera/Screen-depth 변환과 −Z camera/screen-Y-down 의미를 명시했다.
Legacy adapter는 별도 source importer와 제한된 render consumer이며 기존
Core/runtime/project/render2d를 수정하지 않는다. Ground/building geometry와
upright feet를 composition 전에 투영한다. 완성 framebuffer를 warp하지 않는다.

## Public API

추가: ge4g-pentomino-view, ge4g-pentomino-view-legacy. ViewHost lifecycle,
CameraId/ViewId/FormatId, CameraTarget/Viewport/Projection, explicit transforms,
RecordBinding/FormatPlugin, composable follow/bounds/zoom/shake/TransitionMode,
RenderFrame와 structured discovery. `CameraConfig::new` 기본 Smooth12ticks;
Instant 명시 가능. retarget는 직전 committed base에서 시작하는 C0 연속성이다.
Quaternion shortest NLERP는 일정 angular speed/C1을 보장하지 않는다.

P1/P2 public API, Core save2, legacy schema/save/ABI1은 그대로다.
정확한 계약: P3_CONTRACT.md; 공개 소비자 안내: ../public/pentomino-p3/.

## Views

Classic2D/TopDown은 X/Y follow, Side는 View-X, Vertical은 View-Y 정책이다.
공통 behavior를 조합하며 별도 Format과 policy로 설치/교체한다. 한 View의
Camera들도 각자 transform/viewport/transition을 소유한다. Full split-screen UI,
minimap UI, new renderer/client 배포는 구현하지 않았다.

## Tests

실제 명령/결과 원본: evidence/p3/internal-checks.json 및 *.log.
- cargo fmt --all --check: PASS.
- cargo clippy --locked --workspace --all-targets -- -D warnings: PASS.
- cargo test --locked --workspace:150PASS/0FAIL/0ignored, 새 독립34개 포함.
- headless CLI clippy/test, P2/P3 boundary scanners: PASS.
- 기존5demo test --json: replay determinism/save_reload/golden/PNG 전부 PASS.
  기존 Nuvema1957ticks RGBA9a57d2fa… 유지.
- public p3_public/fifth_demo examples: PASS;82실제entities, frozen Core hash
  d94b70a2… 유지, smooth midpoint save/restore output identical, 제거/재설치 동일.
- release consistency, Python unittest, pentaworks validator: PASS.
- FILETREE update/lint: PASS at candidate integration.

기존 P1/P2 테스트는 삭제·약화·변경하지 않았다. Linux-hosted Rust 인프라 검사이며
Flutter/Dart 및 실제 Android/Windows device 검사는 UNVERIFIED이다.

## Defects Found

설계 감사: valid 다른 Scene 레코드의 필터링, source importer 추가 record budget,
legacy action namespace 충돌, World/View follow 이중 mask, float restore 재정규화,
legacy 정수 arithmetic 안전 범위를 구현 전에 수정했다.

실제 구현 결함과 수정:
1. 정상 legacy hp:null을 거부하던 validator 수정; 같은 fifth 테스트 PASS.
2. smooth ortho→ortho 중간 Blended weight0을 호환 renderer가 거부하던 경로 수정.
3. 잘못된 CameraFrame Scene 연결을 명시 거부하도록 수정.
4. valid large-near minimum-width lens 전환에서 rounding으로 interval이 .001
   미만이 되어 실패. public repro tick8/independenttest tick2 실패 후 positive-width
   interpolation/convex clamp/outward rounding으로4096ticks PASS.
5. View 교체가 현재 pose만 검증하던 경로에서 pending endpoints도 새 정책으로
   검증하여 향후 overflow가 있는 replacement를 atomic rejection한다.

6. 신규 CI 보존 검사에서 shallow checkout에 baseline commit이 없어 실패했다.
   actions/checkout fetch-depth0로 비교 이력을 제공하며 검사를 약화하지 않는다.

7. Source-free legacy example 링크에서 vendored Lua static archive가 빠진
   SDK packaging 결함을 재현했다. Cargo native linked_paths archive도 포함하고
   runner에 native search path를 지정하여 실제 unpacked consumer로 다시 검증한다.

실제 pre-fix repro와 failure: evidence/p3/lens-reproducer.rs,
lens-prefixed-failure.log. 저장된 impossible progress/tick0 tracking 검증도 보강했다.

## Core Boundary Audit

독립 Math/source 및 CoreBoundary/source 감사 PASS; P3_CONTRACT_REVIEW.md.
Core 변화가 필요하다는 blocker는 발견하지 않았다. Source importer가 실제
ordinary Core transaction으로 같은 owner objects/records를 초기화하며 authority
검사를 우회하지 않는다. Camera의 state 변경은 presentation host에 한정된다.

## Known Risks / UNVERIFIED

- 외부 GPT Work는 아직 수행하지 않았다. 패키지 selfcheck는 내부 검사다.
- Linked trusted callback hidden state/외부 부작용 및 process sandbox는 보장 밖이다.
- Arbitrary cross-platform float bit equality는 UNVERIFIED; 동일 target/toolchain 한정.
- Legacy adapter는 orthographic-equivalent/identity rotation의 검증된 subset만 지원.
  일반 native perspective/rotation은 legacy renderer에서 Unsupported로 거부한다.
- Snapshot importer는 current Scene 한 번의 projection이며 continuous legacy gameplay
  delegation이 아니다. 반복 import identity 유지도 주장하지 않는다.
- 큰 custom selection의 entity lookup 비용은 quadratic 가능; device/max8View workload
  및 P2 대비 speed improvement는 UNVERIFIED. P3_PERFORMANCE.md 참고.
- 역사적 Windows ZIP timestamp/digest 간헐 실패의 원인·해결은 UNVERIFIED.

## External Test Package

Source-free compiled artifacts, PUBLIC_API/Camera contract/Core contract, discovery,
consumer examples, 기존 fifth 콘텐츠, runner, MANIFEST/SHA256/exact toolchain를
scripts/package_p3_external.py로 준비한다. Packaging/upload identity와 selfcheck
결과는 candidate evidence에 추가한다. Separate GPT Work가 새로운 consumer test를
작성해야 하며 repo/internal implementation/existing tests를 읽지 않는다.

**P3 외부 테스트가 필요합니다.** ../public/pentomino-p3/EXTERNAL_TESTER_TASK.md를
새 GPT Work 세션에 전달하고 package hash/testsource/rawlogs와 보고서를 반환한다.

## Next

현재 main은 P0/P1, accepted P2 PR3는 OPEN, P3는 pentomino/p2-tiny-core에 쌓인다.
외부 blocking defect를 해결하고 accepted gate를 확인하기 전 P3 완료로 처리하지
않는다. 이후 Gameplay plugin 단계로 넘긴다. Combat/world/interaction/platformer,
Forge/story/asset generation/full renderer rewrite는 여기서 구현하지 않는다.
