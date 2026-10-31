# GE4G P4 Flutter / Android 실기기 검증 보고서

공개본 안내(2026-10-31): 시험은 2026-10-19에 수행했다. [조건·계획·과정과 판정 범위](README.md)를 함께 읽는다. APK와 구현 소스는 이번 공개에 포함하지 않는다. 아래 약 1.665초는 정상 기기 구간의 근사치이며 실패한 logcat 수집 대기 30.190초를 포함한 합계는 약 31.855초다. 저장/복원 PASS는 전투 상태 비교, close PASS는 보조 세션 핸들 무효화 범위다. 화면 주 세션의 앱 종료 close는 미검증이다.

## TL;DR

판정은 **PARTIAL PASS — 테스트 하네스 범위만 PASS**다.

Samsung SM-S918N(API 36, arm64-v8a)에서 Flutter 디버그 하네스가 실제 P4 ARM64 `cdylib`를 로드하고 공개 C ABI v1을 호출했다. 세션 생성, 구조화 관찰, 고정 피해 공격, 두 Core tick, canonical RGBA 프레임 표시, 저장/복원, close 후 핸들 무효화, 백그라운드 시 held 터치 해제가 모두 통과했다.

그러나 현재 제품 Flutter 앱에는 P4 연결이 없다. P4 handoff section 7도 Android playable-client wiring을 후속 범위로 남긴다. 따라서 이 결과를 제품 앱 P4 지원, 서명 릴리스, 사용자 수용 또는 P4 최종 릴리스 승인으로 해석하면 안 된다.

## 실제 관찰

- 기기: `SM-S918N`, serial `R3CW208CWZH`, Android API 36, `arm64-v8a`.
- 하네스 패키지: `dev.ge4g.ge4g_client.p4harness`, debug certificate, 생산 앱과 side-by-side 설치.
- 생산 패키지 `dev.ge4g.ge4g_client`의 설치 경로는 전후 그대로 확인됐다. 앱 데이터 내용은 읽지 않았다.
- 네이티브: ELF64 / AArch64 / DYN, ABI version 1.
- 초기 전투: HP `[12,11]`, turn 0, attack_count 0.
- 1회 합법 공격: HP `[12,7]`, turn 1, attack_count 1, core_tick 2.
- 저장 후 두 번째 공격, 복원 결과: HP `[12,7]`, turn 1, attack_count 1로 정확히 복귀.
- 프레임: 160×96 RGBA8, 61,440 bytes, SHA-256 `b21c52050cf1052af6cbf55fc46f367d1828fcd42ebdf995f17b6e92fd28ebb4`.
- 별도 세션 close 뒤 observe는 `invalid_session`으로 거부.
- 터치 DOWN 상태에서 HOME→복귀 시 `released_held_input=true`, session alive, attack_count 1 유지. 즉 백그라운드 전환이 공격으로 오인되지 않았다.
- Flutter 화면에서 실제 canonical frame과 PASS 상태를 확인했고 `harness-pass.png`로 보존했다.

## 판정표

| 항목 | 판정 | 근거/한계 |
|---|---|---|
| 제품 Flutter 앱의 P4 통합 | **NOT DELIVERED** | `client`에 기존 P4 FFI wiring이 없고 handoff가 후속 작업으로 명시 |
| Flutter→ARM64 P4 native load | **PASS** | 실기기 ABI 1, AArch64 `.so`, app-scoped log |
| P4 세션/구조화 observation | **PASS** | 초기 HP/turn/count 실측 |
| 합법 action/고정 피해/2 tick | **PASS** | `[12,11] → [12,7]`, turn 1, count 1, core_tick 2 |
| canonical frame 전달·표시 | **PASS** | RGBA 크기·SHA-256 일치, Flutter 화면 캡처 확인 |
| save/restore 동등성 | **PASS** | 공격 후 저장 상태와 복원 observation 일치 |
| close/핸들 무효화 | **PASS** | close 후 `invalid_session` |
| 하네스 held 입력 background release | **PASS** | HOME/복귀 후 released true, count 불변 |
| 제품 P4 touch/lifecycle 경로 | **NOT DELIVERED** | 제품 P4 입력 경로 자체가 없음 |
| release signing/배포 APK | **NOT TESTED** | 디버그 하네스만 사용, 개인 키 미접근 |
| 사용자 물리 수용 | **NOT TESTED** | 자동/관찰 검증이며 사용자 수용을 대신하지 않음 |
| blind consumer/P4 최종 릴리스 | **NOT TESTED / WITHHELD** | 기존 release hold와 독립 문제 |

## 시간

- 첫 ARM64 네이티브 빌드: 99.447초.
- 첫 Flutter APK 빌드: 95.155초.
- 두 빌드 합계: 194.602초.
- Rust 타깃 준비 + 격리 wrapper 생성: 10.887초.
- 설치: 9.298초.
- Android cold launch TotalTime: 1.373초.
- 의도적 기기 시나리오 대기: 약 1.665초(핵심 결과 약 1.061초 + lifecycle 0.604초).

빌드/다운로드 시간과 테스트 대기 시간을 분리했다. 기존 7200초 endurance, 전체 workspace, 에뮬레이터 검증은 반복하지 않았다.

## 산출물 무결성

- APK: `ge4g-p4-harness-debug.apk`, 77,806,394 bytes, SHA-256 `597aeb4508bbbae99c459ef6df228889208bc8965f8e5882462b9aff51c3e4dd`.
- APK 내부 P4 `.so`: 2,320,288 bytes, SHA-256 `5484e96049204fa54f4fca7547110d1b0f4b5fe8e4b4e5167fd7561fd562af9f`.
- 로컬 빌드 P4 `.so`: 3,786,472 bytes, SHA-256 `4a4dda5350d07bad9f29bca5b5091127005a4765913d4819efd3c0ccda8af2ee`.
- 화면 캡처 SHA-256: `af0b335645775f2b991a831ed432008c72108b4ba62610163ad68b5908164747`.

APK 내부 `.so`는 Android 패키징 과정에서 strip되어 로컬 빌드 파일과 크기/해시가 다르며, APK 엔트리 자체를 별도로 해시했다.

## 시행착오와 판단 근거

- 등록된 client는 Dart `>=3.13.0`을 요구하지만 로컬 Flutter 3.44.4는 Dart 3.12.2였다. 공유 SDK를 업그레이드하거나 제품 SDK 제약을 낮추지 않고, 동일 추적 Dart 소스와 동일 ARM64 `.so`를 쓰는 격리 wrapper로 전환했다. 제품 Android Gradle 파일은 최종적으로 변경하지 않았다.
- 초기 lint의 FFI `Size` 이름 충돌과 불필요 import는 추적 하네스 소스에서 수정 후 최종 analyze PASS를 확보했다.
- 첫 logcat 수집기는 PowerShell 예약 변수 `$PID` 때문에 잘못된 호스트 PID를 사용했다. 이미 실행 중인 동일 앱의 실제 PID 6726을 찾아 app-scoped 버퍼에서 PASS 마커를 복구했다. 본 시나리오는 재실행하지 않았다.
- 결과는 하네스와 제품 통합을 분리한다. P4 전투 규칙을 Dart로 재구현하지 않았고, 반환된 RGBA를 그대로 표시했다.

## 남은 한계

제품 Flutter 앱에 P4 session/game selection/input/save UI를 연결하는 작업은 별도 수직 슬라이스가 필요하다. 또한 적정 Flutter SDK(Dart 3.13+)로 실제 client 프로젝트 자체를 빌드·설치해 검증해야 한다. 이번 디버그 하네스는 기기에 남아 있으며 생산 앱을 덮어쓰거나 제거하지 않았다.

요청 모델 메타데이터는 dispatch receipt에 `gpt-5.6-sol`, reasoning `high`로 기록됐고 컨트롤러는 `current Codex session`으로 host-claim했다. 서버 측 실제 실행 모델/과금 정보는 이 작업에서 독립 조회할 수 없어 추가 추론하지 않았다.
