# P4 Flutter Android 실기기 시험 기록

시험일: 2026-10-19 (Asia/Seoul). GitHub 기록 정리: 2026-10-31.

이번 변경은 이미 수행한 시험의 조건, 계획, 실제 과정과 결과를 공개한다. 구현 소스, CONTINUUM 컨트롤러 설치 파일, 로컬 작업 원장, APK는 포함하지 않는다. 2026-10-31에 시험을 다시 실행하지 않았다.

## 목적과 조건

- P4 NativeBridge가 Flutter를 통해 Android ARM64 실기기에서 실행되는지 확인한다.
- 대상 소스 기준점은 `1d9640200b2da2e790f5d2d25f7590deac10186e`다. 당시 P4 구현을 포함한 QA 기준점이며 이후 P5/pre1P6 검증으로 확대하지 않는다.
- 요청 실행 환경은 Codex 앱의 `gpt-5.6-sol` / reasoning `high` 세션이다. `dispatch.json`은 요청과 생성 응답의 기록이며 서버 내부 실제 모델이나 과금의 독립 증명은 아니다.
- 기기는 Samsung SM-S918N, Android API 36, arm64-v8a다. Flutter 3.44.4 / Dart 3.12.2, Cargo 1.99.1, Android NDK 28.2를 사용했다.
- 기존 제품 앱 ID와 서명 계약, 저장 데이터를 보존한다. 하네스 ID는 `dev.ge4g.ge4g_client.p4harness`이며 별도로 신규 설치했다.
- 기존 네이티브/에뮬레이터/7200초 endurance 결과는 배경 근거로만 참조한다. 이번 시험 횟수에 합산하거나 재실행하지 않는다.
- 첫 빌드·다운로드 시간과 기기 시나리오 대기를 구분한다. 시나리오는 각각 60초 이내, 합계 의도적 대기는 3분 이내를 목표로 했다. 공개 계약 변경, 데이터 손상 위험, 필요한 기기 권한 부재, 두 번의 독립적인 실패 시도는 중단 조건이었다.

## 계획과 실제 수행

| 계획 | 실제 수행 및 관찰 |
|---|---|
| Flutter/P4 연결 여부 확인 | 당시 제품 Flutter 앱에 P4 ABI 연결이 없음을 확인했다. P4 handoff section 7도 Android 연결을 후속 작업으로 남겼다. |
| 기존 C ABI를 사용하는 최소 하네스 구성 | 시험용 Dart 진입점과 별도 패키지를 구성했다. Rust가 전투를 수행하고 Flutter는 반환된 RGBA를 표시했다. |
| ARM64 네이티브·Flutter 빌드 | 기존 P4 crate를 AArch64로 빌드했다. 제품의 Dart 제약과 로컬 SDK가 맞지 않아 격리 Flutter wrapper를 사용했다. |
| 상태·행동·프레임·저장·종료 검증 | ABI 1, 초기 HP, 1회 공격, 두 Core tick, 프레임 해시, 저장/복원 후 전투 상태, 보조 세션 close 후 핸들 거부를 검사했다. |
| 짧은 background/resume 시나리오 | 터치 DOWN → HOME → 앱 복귀 → UP을 수행했다. held 입력 해제, 세션 생존, attack_count 불변을 관찰했다. |
| 증거와 제한 기록 | app PID로 제한한 logcat, 명령·시간·종료 코드, APK/라이브러리 해시, 화면 캡처, 결과 JSON을 남겼다. T-0002를 host-submit했고 독립 finish는 대기 상태였다. |

## 결과와 판정 범위

**PARTIAL PASS: 별도 하네스의 Flutter/ARM64 실기기 실행이 통과했다.** 제품 Flutter 앱 P4 통합은 `NOT DELIVERED`다. 제품 앱 수용, release signing, blind-consumer 인증, 최종 P4 릴리스 승인은 이 시험으로 충족되지 않았다.

공격 뒤 HP는 `[12,11]`에서 `[12,7]`로 바뀌었고 turn 1, attack_count 1, core_tick 2였다. 표시한 프레임은 초기 canonical RGBA 160×96이다. 저장/복원 검사는 전투 상태의 동등성을 확인했으며 전체 Core/View 저장 바이트와 프레임의 복원 동등성까지 검사한 것은 아니다. close 검사는 별도로 만든 보조 세션에 대해 수행했다. 화면의 주 세션이 앱 종료 시 정상 close되는 경로는 입증되지 않았다.

## 실패와 기록 정정

- 최초 client 분석은 SDK 제약 때문에 의존성 해석에서 중단됐다. 격리 wrapper로 전환했고 FFI 이름 충돌과 import 경고를 수정한 후 최종 분석이 통과했다.
- 최초 logcat 수집은 PowerShell 예약 변수 `$PID` 때문에 호스트 PID에 연결되어 30.190초를 소비했다. 같은 실행의 실제 앱 PID 6726 버퍼에서 결과를 복구했다. 핵심 시나리오를 다시 실행하지 않았다.
- 최초 화면 캡처는 셸 인용 오류로 실패했다. 바이너리 stdout 리디렉션으로 같은 앱 화면을 수집했다.
- 제품 Gradle 설정의 추가 확인은 wrapper 부재, 대체 wrapper의 Gradle 9.1/프로젝트 요구 9.3.1 불일치로 끝났다. 제품 Gradle 변경은 원복했고 제품 프로젝트의 전체 빌드 성공으로 보고하지 않았다.
- 첫 네이티브/Flutter 빌드는 99.447초/95.155초였다. 기존 보고서의 약 1.665초는 정상 기기 결과 구간의 근사치다. 실패한 logcat 수집 대기까지 포함하면 약 31.855초이며, 전체 작업 시간이나 전체 도구 실행 시간을 뜻하지 않는다.

## 공개 증거

- [한국어 상세 보고서](REPORT.ko.md)
- [기계 판독 결과와 산출물 해시](result.json)
- [명령·시간·종료 코드](COMMANDS.md)
- [의사결정 기록](DECISIONS.md)
- [모델 요청 기록](dispatch.json)
- [핵심 시나리오 app-scoped 원본 로그](device-logcat-recovered.raw.log)
- [background/resume app-scoped 원본 로그](device-lifecycle-logcat.raw.log)
- [실기기 화면](harness-pass.png)
- [공개 파일 SHA-256 목록](SHA256SUMS.txt)

APK 파일은 이 문서 공개에 포함하지 않는다. 보고서와 JSON의 APK 경로는 시험 당시 로컬 산출물 경로이고, 크기와 SHA-256은 이력으로 보존한다. 구현 하네스와 격리 wrapper 또한 이 PR에 없으므로 이 기록만으로 앱을 재빌드할 수는 없다. 빈 수집 로그 네 개는 공개 목록에서 제외했다. 원본 로그의 바이트는 변경하지 않았다.
