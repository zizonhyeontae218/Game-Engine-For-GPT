# Rundown — P4 독립 QA / 2026-10-16

**TL;DR:** 지정 CI SDK의 독립 검증 24/24 그룹 PASS, 재현된 엔진 결함 0개다. 새 로컬 빌드에서도 13/13 그룹, 전체 Rust 169/169, 별도 headless CLI 9/9를 직접 실행했다. Linux P4 기능 범위는 승인 가능한 근거를 갖췄다. 플랫폼·계측·접근 격리 인증까지 포함한 무조건 최종 승인은 보류한다.

## 상태와 집계 기준

| 직접 실행한 대상 | PASS | FAIL | 증거 |
|---|---:|---:|---|
| 고정 CI SDK 독립 그룹 | 24 | 0 | independent_results.json, extended_results.json |
| 새 소스 빌드 검증 그룹 | 13 | 0 | local_source_results.json |
| Rust workspace 개별 테스트 | 169 | 0 | source-checks/workspace-tests-resolved.log |
| no-default-features CLI 개별 테스트 | 9 | 0 | source-checks/headless-cli-tests.log |
| 별도 C 응답 소유권 검사 | 1 | 0 | ownership.log, 302개 응답 해제 |

그룹과 개별 테스트는 서로 다른 집계 단위다. 로컬 13그룹 중 C 링크 그룹은 CI SDK를 링크하고, 나머지 행동 검사는 새 로컬 cdylib를 호출한다. P4 Rust 19개 테스트는 workspace 169개에 포함된다. 기존 감사 결과를 신규 실행 수에 더하지 않았다.

추가 직접 실행 PASS: fmt, workspace/headless strict Clippy, release consistency self-test, P2/P3/P4 경계 검사, P3/P4 공개 예제와 fifth 렌더 예제, 기존 5개 게임 replay/golden/save 검증. 원본 명령과 종료 코드는 source-checks/*.json 및 로그에 보존했다.

## 검증 대상과 무결성

- PR #5 기준 소스: `72d49a63f730e2a3ee2032e83948894e4fd480ba`.
- CI run `38414825262`, artifact `11823886424`.
- SDK manifest commit `62b7118a477d40341bb8a64221d0adca70dddbce`는 CI synthetic PR merge checkout이다.
- 외부 artifact ZIP SHA256: `af98895c3baf92fb502b13951b056d85ad558f5eb4c915b6d5177c41cbf8600c`.
- SDK ELF SHA256: `c5f81138a559154e287c0710daf497b04484f6cc37906a51ddc5713b75e8b822`.
- 내부 ZIP 체크섬, 모든 manifest 파일 크기·SHA256, C11 compile/link/runtime를 직접 확인했다.
- 실제 final zoom/plugin-cycle RGBA: `6ef9dc3aaff45923a9a346d1fd7e71e0dbdef9efc0c1541d7654751f61f9f0f4` 재현. 기존 이미지와 신규 픽셀 출력도 확인했다.
- 환경: Linux x86_64, Python ctypes/cc, SDK rustc1.99.0 debug; 로컬 rustc1.99.1. 상세 환경/원본 ZIP 해시는 environment.json.

## 직접 검증한 내용

기존 14그룹: ABI envelope/discovery/frame, 수계산 전투 oracle, 결정론, 오류 원자성, 잘못된 저장 거부, 실제 규칙 제거·재설치, Camera/View, stale handle/16개 한도, 스레드 귀속, cross-mode 복원, 24개 무작위 전투 설정, 80개 malformed bytes.

확장 10그룹: 정확히 24MiB인 유효 요청 수락·1byte 초과 거부, 깊은 JSON·중복 필드 거부, 21개 invalid config 및 512×512 RGBA/최대 seed·ID·control, 19개 저장 변조 원자성, 추가 2,000개 random byte/500개 구조적 invalid input, 8스레드에서 동시 128세션과 800스텝, 4프로세스 동일 save hash, 300회 공격과 256개 이력 유지 경계·복원, Headless/Linked 각 5,000스텝, 1,001세션 생성·종료, 128개 camera/View 변형.

1만 스텝 동안 관측한 RSS는 44,300KiB로 일정했고, 세션 churn 이후 증가도 0KiB였다. 이는 해당 프로세스의 관측으로 누수 부재의 증명은 아니다. no-op p50은 Headless 약7.36ms, Linked 약6.55ms였다. 80.5초 반복 검사이며 수시간 soak가 아니다. 공유 호스트의 동시 빌드와 debug SDK 때문에 기기 성능·성능 향상·실시간 SLA로 해석하지 않는다. 300회 공격 검사는 약26.7초로, no-op 수치를 긴 전투 비용으로 일반화할 수 없다.

## NOT TESTED — 7개 검증 게이트

| 대상 | 이유 |
|---|---|
| Windows P4 실제 클라이언트·실기기 | 해당 플랫폼 실행 환경 없음, P4 클라이언트 미제공 |
| Android P4 실제 클라이언트·실기기 | adb/기기/에뮬레이터 없음, P4 클라이언트 미제공 |
| ASan 계측 실행 | 2회 계측 빌드에서 rlib의 zero-length object mapping 오류; 테스트 실행 전 차단 |
| Valgrind 실행 | 2회 시도 모두 loader strlen 심볼 redirection에서 시작 실패 |
| TSan 실행 | 실행하지 않음; 일반 동시성 테스트와 구분 |
| 접근까지 차단한 blind consumer 인증 | 테스트 프로세스는 SDK만 사용했지만 QA 에이전트는 소스 접근 가능 |
| 수시간 endurance | 실제 측정은 1만 스텝/약80.5초 |

잘못된 C 포인터, OOM/process crash 복구와 저장 진본 인증은 v1 보장 밖이며 승인하지 않았다. sanitizer 도구 실패를 엔진 FAIL이나 PASS로 바꾸지 않았다.

## 기존 로그만 확인한 내용 / 문서 차이

GitHub client CI run38414825290의 Windows·Android 성공은 커넥터로 확인했다(ci-platform-jobs.json). 기존 FlatLand Flutter/ABI1 클라이언트 검사이며 P4 네이티브 브리지의 실제 플랫폼 실행 증거가 아니다. 원본 감사 ZIP의 과거 14/14·500스텝 결과는 prior-*에 별도 보존했다.

Drive 검색과 실제 읽기: drive-review.md. P3_RUNDOWN.md의 예전 external pending 및 Soul.md의 FlatLand0.2 상태는 역사적 기록이다. 현재 GitHub의 P2/P3 병합·사용자 수용 기록과 P4 계약을 우선했다. Drive 역사 문서는 수정하거나 삭제하지 않았다.

## 결함과 호환성

재현된 엔진 결함 0개, 제품 코드 수정 0개다. 따라서 결함 수정용 최소 재현/전후 비교는 해당 없음이다. 이번 PR은 검증 증거만 추가한다. source-integrity.json에서 이전 Core/View/FlatLand/client/examples의 기준 대비 변경 0개를 확인했다. Android 앱 ID·인증서 파일은 보존했으며 실제 APK 서명 검증을 새로 했다고 주장하지 않는다.

초기 환경 문제(Rust/rustfmt/pkg-config/ALSA 부재), 계측 빌드 실패, 작업 중 FILETREE stale 로그도 보존했다. 도구 설치/로컬 의존성 추출로 일반 빌드와 회귀를 완료했고 검사 조건을 약화하지 않았다.

## 재현

CI artifact를 내려받아 내부 SDK ZIP과 checksum 파일을 추출한 뒤 SDK를 별도 디렉터리에 푼다. 이 폴더에서 다음을 실행한다. 경로는 실제 절대 경로를 사용한다.

```sh
export GE4G_P4_SDK=/absolute/work/sdk
export GE4G_P4_OUTER=/absolute/work/ci-artifact.zip
export GE4G_P4_ARCHIVE=/absolute/work/p4-public-sdk.zip
python3 independent_p4_test.py
python3 extended_p4_test.py
```

테스트 모듈은 엔진 Rust 소스나 SDK consumer.py를 import하지 않는다. baseline 결과14그룹 + extended 결과10그룹을 합쳐24그룹이다. 새 소스 빌드 검사는 별도 local_source_test.py와 local_source.log를 참조한다. 원본 감사 ZIP과 다운로드한 CI artifact는 별도 최종 증거 번들에도 보존한다.

## 정리와 판정

영구 증거와 Issue #6 반영 후 추출된 임시 WORK_HANDOFF.md만 삭제한다. 공식 docs/pentomino/P4_HANDOFF.md와 원본 감사 ZIP은 보존한다. 실행 완료 여부는 cleanup.json/Issue 정리 댓글을 따른다.

Linux P4 기능 검증은 PASS다. 플랫폼/계측/엄격한 blind 인증을 포함한 최종 포괄 승인은 보류한다. PR #5 병합·정식 릴리즈·P5 기능 개발은 수행하지 않았다.
