# Rundown — P4 남은 검증 / 2026-10-18–19

남은7게이트 중 **6 PASS, 1 NOT TESTED**. 신규 실행에서 확인한 엔진 결함0개,
제품 코드 수정0개다. 접근 격리된 독립 소비자 인증 F는 완료하지 못했으므로
P4의 포괄 최종 수용·릴리즈 승인은 보류한다. 기존 검증 기록은 보존했다.

## 신규 실행과 집계

| 게이트 | 실제 실행 | 판정 |
|---|---|---|
| A Windows | x64 DLL + Windows Python ctypes9/9 그룹 | PASS 네이티브 범위 |
| B Android | Android16/API36 x86_64 에뮬레이터, NDK 최소 C 소비자7/7 그룹·294 ABI 호출 | PASS 에뮬레이터 네이티브 범위 |
| C ASan | nightly1.101.0 + rebuilt std, Rust/실제 FFI20/20 | PASS |
| D Valgrind | 원본 CI ELF, 기존 C 소유권302응답 +8스레드128응답/세션 종료 | PASS2/2 프로세스 |
| E TSan | nightly1.101.0 + rebuilt std, Rust/실제 FFI20/20;8스레드128개 전용 세션 | PASS |
| F 접근 격리 blind | 신규 에이전트의 도구·파일 읽기 권한을 제한할 인터페이스 없음 | NOT TESTED |
| G endurance | Headless7200.109초/68,972스텝, Linked7200.032초/67,608스텝 | PASS2/2 실행 |

그룹·개별 테스트·프로세스·장시간 실행은 서로 다른 집계 단위다. 최종 sanitizer
40개와 별도로 **첫 실행에서도40개가 통과**했지만 테스트 뒤 해시 수집 단계에서
2개 job이 실패했다. 따라서 sanitizer 실제 실행은80회 PASS/테스트 FAIL0이며,
최종 고유 테스트는 ASan20개·TSan20개다. 첫 실패를 숨기거나 엔진 실패로 분류하지 않았다.

## 실제 범위

Windows에서 생성/입력/관측/골든 RGBA/전투/저장복원/규칙 제거·재설치/
Camera·View와 실패 원자성/종료·stale handle/스레드 귀속성을 실행했다.
Android에서는 같은 공개 ABI의 전투·실제 RGBA·저장복원·규칙·Camera·View·
실패 원자성·수명주기를 검증했다. Android 소비자는 한 네이티브 스레드에서
호출하므로 Android 스레드 귀속성까지 시험했다고 주장하지 않는다.

기존 AVD Medium_Phone_API_36.1을 WHPX, x86_64,1024MiB,2cores,
read-only/no-snapshot으로 실행했다. Rust Android 타깃의 최소 API는26이다.
APK를 설치·제거하지 않았고 앱 데이터·앱 ID·인증서·키를 변경하지 않았다.
기존 client에는 P4 심볼 연동이 없다(source-integrity.json).
**Flutter UI, Windows 배포 앱, Android ARM/물리 기기는 NOT TESTED**다.

Sanitizer는 Rust 의존성·std·P4와 실제 exported FFI 함수가 계측된 범위다.
vendored C Lua와 system libc 전체 계측을 주장하지 않는다. TSan은 서로 다른
스레드의 독립 세션·전역 handle 할당을 검사하며 공유 세션의 계약 밖 사용은 없다.
Valgrind3.18.1 두 실행 모두 **종료 힙0bytes/0blocks, errors0, suppressed0**이다.
definitely/indirectly lost와 still reachable 모두 해당 실행의 종료 시0이다.
이는 임의 입력·모든 플랫폼에서의 완전한 무누수 증명이 아니다.

## Endurance 부하와 관측

각 모드 별도 프로세스에서 연속2시간, 합계 **136,580스텝**을 실행하고 종료 코드0을
회수했다. 최대 HP10000/최소 피해1, 독립 전투 oracle을 매 스텝 대조했으며
각 모드에서19,999회 공격의 긴 전투를 완료했다.256개 이력 유지 경계 이후도 계속
실행했다.11번째마다 빈 입력을 섞고, 전투 종료 뒤 새 전투를 시작했다.

| 모드 | 저장복원 | 종료·재생성/새 전투 | Camera/View | 규칙 제거·재설치 | RGBA 검증 |
|---|---:|---:|---:|---:|---:|
| Headless |1111|37|543|135|frame Unsupported가 계약 |
| Linked |1089|36|532|132|4225|

상태·2Core tick 경계는 매 스텝 검사했다. 저장복원은 canonical save 동일성,
Camera/View는 전체 Core save 불변 및 잘못된 View의 원자성, 재생성은 stale handle과
동일 저장 복원, 프레임은 크기/RGBA/SHA256을 검사했다. 종료·재생성 횟수에는
완료 전투 뒤 초기화가 포함되며 저장 복원과 구분한다.

원본30초 체크포인트에는 스텝·RSS·최근4096회 작업 지연 p50/p95/p99·오류·상태
해시가 있다. 최종/최소·최대 RSS와 체크포인트 간격은 summary.json 참조.
작업 지연에는 해당 반복의 assertion·프레임·저장·수명주기 작업도 포함되며
게임 FPS나 순수 엔진 호출 SLA가 아니다. 각 프로세스 RSS768MiB, 최대50회/초 상한을
두고 호스트 여유512MiB 미만이 두 번 지속되면 QA 프로세스만 중단하도록 감시했다.
자원 중단·반복 크래시·연속성 단절은 없었다. 사용자 지시 이후 진행 읽기는5분 간격으로
줄였고, 테스트 자체는 재시작하지 않았다.

## 준비 실패·QA 결함과 조치

- WSL은 설치되어 있지 않았다. OS/WSL 신규 구축 대신 공개 저장소의 표준 무료
  Ubuntu22.04 GitHub runner를 사용했다. 기존 Android SDK/NDK/AVD를 사용했다.
- 과거 ASan zero-length rlib mapping 오류는 이번 정상 Linux/nightly 환경에서
  재현되지 않았다. 이전 오류의 정확한 원인을 엔진 문제로 확정하지 않았다.
- Valgrind는 matching libc6-dbg와 정상 loader로 실행했다. 과거 strlen redirection
  시작 실패를 해결했으며 custom/broad suppression·보안 설정 완화는 하지 않았다.
- **QA 결함1개 수정:** nightly Cargo가 테스트 바이너리를 debug/build/.../out에
  배치해 기존 debug/deps 한정 해시 수집이 실패했다. `33b8193`에서 경로 수집을
  수정했고 ASan/TSan만 한 번 재실행해 성공·해시를 확보했다. Valgrind는 반복하지 않았다.
- F: 현재 spawn/thread 도구에 소스 읽기/도구 allowlist 제한 설정이 없어 독립 소비자
  에이전트의 접근 격리를 보장할 수 없다. 소스를 보지 말라는 지시나 out-of-tree
  실행을 격리 인증으로 재명명하지 않았다.

## 원본·재현·보존

구현 `72d49a63f730e2a3ee2032e83948894e4fd480ba`, QA
`7258d48e1b02ec2c8bdca9099267bd17da924ecc` 이후 원격 추가 변경은 시작 시 없었다.
제품·고정 의존성·P0–P3·기존 게임·앱·공식 계약/인계·이전 QA 증거의 변경0개를 확인했다.
원본 artifact11823886424는 만료되지 않았으며 외부 ZIP
`af98895c3baf92fb502b13951b056d85ad558f5eb4c915b6d5177c41cbf8600c`, ELF
`c5f81138a559154e287c0710daf497b04484f6cc37906a51ddc5713b75e8b822`를 검증했다.
manifest의62b7118a는 synthetic merge commit이다.
Windows/Android와 sanitizer는 새 소스 빌드, Valgrind는 원본 ELF 검증으로 구분한다.

- `COMMANDS.md`: 명령·도구·플랫폼·부하·제약.
- `windows-native.*`, `android-native.*`, 빌드 로그·getprop·environment: 실제 플랫폼 증거.
- `linux-first/`: 첫 실행·해시 수집 실패·성공한 Valgrind 원본 로그.
- `linux-final/`: 수정된 sanitizer 원본 로그와 테스트 바이너리 해시.
- `endurance-*.jsonl`, `*.stdout`, 최종 JSON, `host-monitor.jsonl`: 원본 장시간 기록.
- `artifact-integrity.json`, `source-integrity.json`, `process-exits.json`, `cleanup.json`,
  `SHA256SUMS.json`: 무결성·종료·보존 기록.

Linux 첫 run[39138455495](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/39138455495),
수정 run[39138832654](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/39138832654).
도구 버전과 실행 커밋은 각 driver/run JSON에 보존했다. 신규 raw evidence에는
Git newline 변환을 꺼 원본 바이트와 SHA256을 유지한다.

이미 완료된 SDK24/24·로컬13/13·workspace169/169·CLI9/9·기존5게임/fmt/strict
Clippy/경계 검사는 **이전 기록만 확인**했으며 신규 집계에 더하지 않았다.
자동으로 발생한 중복 전체 acceptance job은 취소했고 최종 증거 커밋은 skip-ci로
불필요한 전체 재검사를 막았다. FILETREE update/lint와 커밋 checkout 일관성은 별도 확인한다.
Drive 역사 검색도 이전 drive-review를 활용했다. 사용자 지시 이후 고정 worktree 밖
저장소 탐색·신규 대규모 환경 구성은 하지 않았다. WORK_HANDOFF를 재생성하지 않았다.

## 계약에 따른 최종 판정

1. **PR #5 코드 병합: 기술적으로 조건부 가능.** 기능·수명주기·안전성·장시간 부하에
   새 엔진 결함은 없다. 다만 P4_RUNDOWN의 독립 새 세션 소비자 체크와 GATES G5의
   공개 소비자 수용은 포괄 완료로 선언할 수 없다. F의 미완료를 명시한 실험적 P4
   코드 검토/병합 근거이며, 무조건적인 P4 최종 수용 승인은 아니다.
2. **플랫폼 실행 승인: Windows x64·Android x86_64 에뮬레이터의 최소 네이티브
   소비자 범위만 PASS.** P4_HANDOFF 항목7과 P4_CONTRACT의 마지막 절에 따라
   Flutter/배포 앱·ARM/실기기 전달은 후속 playable/platform 단계다. 이를 P4의
   모든 기능 실패와 동일하게 취급하지 않되 플랫폼 앱 실행을 승인하지 않는다.
3. **릴리즈 승인: 보류.** 독립 소비자 인증이 남았고 플랫폼 앱·기기·패키징/서명
   수용 증거도 없다. PR 병합·정식 릴리즈는 수행하지 않았다.

후속[PR #8](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/pull/8)은
PR #7 QA 브랜치를 대상으로 하며 구현 PR #5와 Issue #6에 연결한다.
