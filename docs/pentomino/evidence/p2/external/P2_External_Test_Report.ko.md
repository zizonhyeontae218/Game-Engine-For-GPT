# Pentomino P2 외부 소비자 테스트 보고서

## TL;DR / Rundown

- **외부 P2 gate: ACCEPTED — 제공된 source-free Core SDK 범위.**
- 신규 독립 Rust 테스트 **23/23 PASS**. 공개 예제 실행과 SDK 체크섬·컴파일러 검사도 PASS.
- 관찰한 실행 결과에서 gate를 막을 기능 결함은 발견하지 못했다.
- **레거시 adapter/기존 게임 실행은 UNVERIFIED**. 이 승인은 레거시 실행, P3 Camera/View, 내부 구조, 모든 가능한 입력의 증명을 포함하지 않는다.
- 상태: [x] 패키지 무결성 [x] 독립 consumer 실행 [x] 6개 공개 SDK challenge [x] 보고서/소스/원본 로그 [ ] SDK 밖 레거시 실행
- 분기 판정: 공개 Core SDK 외부 검증 통과. 구현 저장소의 병합·버전 표기·배포 상태는 변경하지 않았다.

## 독립성 및 사용한 자료

이번 세션의 새 scratch workspace에 사용자가 제공한 ZIP만 풀었다. 엔진 저장소를 가져오지 않았고 내부 Core/adapter 소스나 기존 구현 테스트를 읽지 않았다. `MANIFEST.json`, `PUBLIC_API.md`, `CONTRACT.md`, `PLUGIN_DISCOVERY.md`, `README.md`, `QUICKSTART.ko.md`, 공개 runner를 읽고 테스트를 새로 작성했다. 제공 예제는 runner로 실행했지만 예제 소스를 재사용하지 않았다. 컴파일된 rlib와 그 의존 라이브러리만 링크했다. 병렬 sub-agent나 이전 구현 세션의 파일을 사용하지 않았다.

승인은 문서나 예제 성공에서 추정한 것이 아니라 아래 신규 consumer assertions의 실제 실행에 근거한다.

## Artifact 및 환경

| 항목 | 기록 |
|---|---|
| SDK artifact | GE4G0.3.0-alpha.1-P2-public-candidate |
| SDK commit | 48d7f4aa6932c9031afee722e0c58f7756b864c8 |
| Archive SHA256 | da0c744914c910479d49a5dbe017526e8366736cede1931515f2e2b851e9c76d |
| Core rlib | lib/libge4g_pentomino-8f6ad56adc69f70b.rlib |
| Core rlib SHA256 | bd0b89a7855ccd5cf6ad8b5839002cbe3f79c38c875e4559e70eba9fe2c8bd35 |
| Contract / save | 2.0.0 / 2 |
| OS | Ubuntu 24.04.3 LTS (Noble), x86_64 Linux |
| Kernel | Linux ec0e2d0f4c86 6.18.44 #1 SMP Sat Sep 26 20:02:31 UTC 2026 |
| Rust / target | rustc 1.99.0 (b940084d7 2026-09-28), x86_64-unknown-linux-gnu |
| Rust commit | b940084d7eb6a299eb4bfeb8e34901bc051e7ac4 |
| LLVM | 23.1.1 |
| Edition | 2024 |

완전한 원본 `MANIFEST.json`, `SHA256SUMS`, `rustc -vV`, OS 출력은 이 evidence bundle에 포함했다. MANIFEST가 지정한 30개 파일의 바이트 길이와 SHA256이 모두 일치했다. 테스트 후에도 원본 SDK 파일은 모두 같은 해시였다.

초기 환경에는 rustc가 없어서 요청한 두 runner 명령이 FileNotFoundError로 실패했다. 공식 Rust 1.99.0 배포 manifest에서 rustc/std를 받아 각각 SHA256을 검증해 설치했다. 설치 중 두 큰 shared library가 불완전하게 복사되어 ELF 로더 오류가 발생했으나, 원본 tar member에서 재복사한 뒤 `rustc -vV`가 SDK MANIFEST와 정확히 일치했다. 이 환경 장애는 최종 실행 전에 해소됐으며 최종 BLOCKED 항목은 없다. compiler 검사를 우회하거나 rlib/runner/SDK를 수정하지 않았다.

공식 다운로드 artifact SHA256:

```text
rustc.tar.xz   77171ba2a0345fdf2abc4fedda55d6de078dae7a68527c28be8c77dcc9604bd5
rust-std.tar.xz 3e58dff2d0b72196b5ea4e90536e174d400de88564a52694686b81e091169933
```

## 실행 명령과 핵심 출력

실제 작업 디렉터리:

```text
/workspace/scratch/a90260208d37/p2_sdk/GE4G-0.3.0-alpha.1-P2-public-48d7f4a
```

```sh
export PATH=/workspace/scratch/a90260208d37/toolchain/installed/bin:$PATH
rustc -vV
python3 run_public.py --check
python3 run_public.py
python3 run_public.py --source external_tests.rs --test
./consumer-output --nocapture
python3 run_public.py --source api_error_probe.rs
```

`run_public.py --check`:

```json
{"checksums":"PASS","compiler":"PASS","commit":"48d7f4aa6932c9031afee722e0c58f7756b864c8","external_gate":"UNVERIFIED"}
```

출력의 UNVERIFIED는 원래 패키지 자체 상태이며, 이번 보고서 판정과 별개다. SDK MANIFEST를 갱신하지 않았다.

공개 예제:

```json
{"ok":true,"contract":2,"ticks":301,"per_owner_retained":256,"dropped":44,"hash":"4d5861d952b99cc1f9aadb86874638870ad17ccbb4ac879f5281b87d7284c65e"}
```

신규 테스트:

```text
running 23 tests
A emitted 38,400; B suffix/dropped/RNG/record unaffected; removal and restored continuation equal
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
```

runner는 `rustc --edition=2024 <absolute-source> --extern ge4g_pentomino=<supplied-rlib> -L dependency=<sdk-lib> -o <sdk>/consumer-output --test`를 실행했다. 각 정확한 전체 명령 및 출력은 `evidence/independent-final.log`에 있다. 상세 discovery와 print 관찰은 `evidence/independent-observations.log`에 있다. 관찰 출력 확보용 `--nocapture` 실행도 23/23 통과했다. 마지막 probe 실행이 consumer-output을 덮어쓰므로 테스트를 다시 실행할 때는 위 test runner 명령으로 재컴파일해야 한다.

## Challenge 판정 및 실제 관찰

| Challenge | 판정 | 실제 검증 |
|---|---|---|
| 1. A/B retention·RNG·정렬·제거·복원 | PASS | A 매 tick 128개 ×300=38,400, B 1개 ×300=300. A retained=256/dropped=38,144, B retained=256/dropped=44. B-only host와 B records/RNG/event payload·owner ordinal·pending이 동일했다(global sequence는 비교에서 제외). A 제거 직전/직후 B selection 완전 동일. active retained events 합산 global sequence 유일·증가 및 tick chronology 확인. owner ordinal=dropped+i와 emitted=dropped+retained 확인. 제거 후 save/restore byte equality, continuation을 B-only 및 fresh restored host와 비교했다. |
| 2. Scene/Entity·권한·수명 | PASS | create/remove/recreate의 새 incarnation, 옛 stable refs 및 opaque handles rejection, cross-host scene/entity handles와 owner token 거부, restore 후 모든 옛 handles/tokens 거부 및 신규 발급 성공. record refs, child links, retained event refs 제거 blocker 검사. unbound read/events/scene/entity 거부. bound foreign scene/entity 삭제 거부. missing binding 및 provider unload blocker 확인. consumer-first unload 뒤 참조/자식/이벤트가 정리되어 provider objects 제거 성공. stale/dead refs rejection 및 same-tick cross-owner reference/removal race의 전체 rollback 확인. |
| 3. typed fields·bounds·poison | PASS | Bool, bounded I64 양끝, 한글 UTF-8 6 bytes, Bytes, Scene/Entity refs, depth4 nested lists와 각 list count 검사. missing/extra fields, wrong tag/type, 범위 초과, dead/stale refs, undeclared schema 거부. 선언 bounds 0/초과 및 depth5 거부. 60×1024-byte string payload의 record는 통과, 64×1024-byte payload는 canonical envelope 포함 64KiB를 넘어 BudgetExceeded. events128 통과/129 거부, commands4096 통과/4097 거부. failed command와 129번째 emit 오류를 callback이 숨겨도 tick 실패 및 byte-identical rollback. |
| 4. actions·privacy·all-owner rollback | PASS | Bool/bounded I64 입력으로 독립 2호스트 replay 저장 결과 동일. next tick 강제, past/duplicate tick, future tick, duplicate/unknown action, wrong type/range rejection. accepted action ordering, missing action absence 확인. bound consumer도 다른 owner action을 읽을 수 없음. 128-action frame 통과/129 거부. owner 제거 후 last_input에서 그 owner actions만 사라지고 target tick 유지. 실패 tick 직전/직후 save byte equality 및 기존 handles 유지; 두 plugins의 records/events/RNG/Scene/Entity/input/tick/identity rollback과 동일 frame retry를 clean host 결과와 비교했다. |
| 5. save·metadata·atomic reject | PASS | save→restore→save byte equality 및 fresh installed host에 restore 후 deterministic continuation. malformed/truncated/8MiB+ saves, noncanonical whitespace, changed content/seed/plugin release/installed descriptor/binding 거부. ordinal/emitted/dropped/next_sequence/purged_retained/input tick/identity/object incarnation 변조 및 save1 버전 거부. 실패 restore 후 bytes/owner tokens/scene handles 유지 확인. |
| 6. public discovery·Core genericity | PASS (공개 표면) | fresh describe는 plugins=[] 및 contract2/save2/scalar1, 수치 bounds를 노출했다. generic 소비자 schemas/objects/actions에 숨은 Camera/View/Gameplay 필수 데이터는 없었다. `arbitrary.data`, `camera.view`, `gameplay.logic` capability labels 모두 의미 해석 없이 정상 install/step/save/restore. 빈 owner selection은 빈 결과, unknown owner selection은 rejection. 내부 dependency topology는 공개 SDK 테스트만으로 인증하지 않는다. |
| SDK 밖 LegacyBridge/게임 실행 | UNVERIFIED | ZIP에 해당 runtime/adapter libraries와 게임 데이터가 없다. 문서나 내부 regression 주장으로 인증하지 않았다. |

## 기능 결함과 최소 public reproduction

**확인된 gate-blocking 기능 결함: 없음.** 모든 최종 assertions 통과. 초기에 테스트 작성자가 예상한 오류 코드가 달라 실패했던 항목은 실패 로그를 그대로 보존했고 아래 계약 해석을 기록했다. SDK를 고쳐 통과시킨 것이 아니다.

`api_error_probe.rs`는 문서의 오류 분류를 명확히 하기 위한 작은 public reproduction이다. 실제 실행:

```text
UTF8 schema max_bytes=6; '한글a' bytes=7 => InvalidRecord; atomic=true
Unknown selection owner => StaleHandle
129-action frame (quota checked before unknown/duplicates) => InvalidInput
```

| 재현 | 기대하는 보장 | 관찰 및 해석 |
|---|---|---|
| String max_bytes=6에 7 UTF-8 bytes인 `한글a` set | 입력 거부, init 전체 rollback | InvalidRecord, byte-identical rollback. 선언 schema 범위 위반은 InvalidRecord로 해석했다. 문서의 포괄적인 byte-quota/BudgetExceeded 문장은 이 경우와 구분을 더 명시할 수 있다. |
| unknown owner selection | 미등록 owner 거부 | StaleHandle. 문서가 exact error code를 명시하지 않으므로 처음 테스트의 InvalidIdentifier 가정은 제거했다. |
| 129-action frame | frame quota 거부 및 rollback | InvalidInput. 최종 독립 test는 **129개의 서로 다른, 등록된 actions**로 같은 거부를 확인했다. probe는 최소 코드용으로 duplicate/unknown actions도 포함하므로 rejection 우선순위 관찰용이다. |

초기 `independent-first.log`는 10 passed / 2 failed, 확장 중 `independent-second.log`는 19 passed / 1 failed다. 수정은 schema size 위반을 InvalidRecord로, oversized input을 InvalidInput으로 분류하고 문서상 미지정 selection code에 특정 값을 강요하지 않는 데 한정했다. 최종 rejection/rollback 보장은 그대로 검사했다.

## Public API clarity / integration friction

- `CONTRACT.md`가 executable schema validator 없이 typed 구조, identity, exact bindings 및 숫자 bounds를 충분히 제공했다. 내부 구현 지식 없이 컴파일 및 consumer 구현이 가능했다.
- 오류 코드 표를 명령별로 정리하면 도움이 된다: **schema-declared field size/count 위반=InvalidRecord**, **canonical record/transaction quotas=BudgetExceeded**, **frame count 위반=InvalidInput**이 실제 관찰됐다. broad quota 표현은 초기 consumer의 잘못된 기대를 유발했다.
- `select` unknown owner의 exact code는 공개 문서에 없다. 실제 StaleHandle은 기록했지만 undocumented exact code를 최종 합격 조건으로 만들지 않았다.
- CONTRACT의 몇몇 제목이 아직 “proposed/FROZEN/runtime UNVERIFIED”로 남아 있어 SDK의 확정 public signatures와 설계 초안의 경계가 흐리다. 실제 rustc 링크로 제공된 선언과 동작을 확인했다.
- Ref의 timeline identity와 runtime opaque handle의 freshness 구분은 특히 restore 후 중요하며, 공개 설명대로 동작했다. `PUBLIC_API.md`의 entity scene scope는 EntityData.scene에서 확인되며 EntityRef 자체에는 scene 필드가 없다.
- 이벤트 refs가 eviction/unload까지 삭제를 막는 것은 실제 동작이다. 높은 retention을 가진 데이터 모델에서 object cleanup을 설계할 때 고려해야 한다.
- 세이브는 matching executable plugins를 먼저 install해야 하며 callback의 외부 hidden state는 Core rollback 대상이 아니다. rollback test의 Cell은 의도적인 failure-injection switch로만 쓰고 durable game state를 저장하지 않았다.
- runner는 정확한 compiler mismatch에는 BLOCKED를 출력하지만 rustc가 아예 없으면 traceback으로 끝난다. 처음 사용자의 환경 진단을 위해 친절한 BLOCKED 출력이 유용하겠다.
- ReadFrame 직접 JSON wire encoding은 문서상 보장 대상이 아니어서 시도하지 않았다. CoreHost.save만 persistence 비교에 사용했다.

## 명시적으로 실행하지 않은 영역

- LegacyBridge/LegacyProjection의 Pac-Man, v1 demo, Harbor Workshop, Signal Yard 및 Unicode legacy source ID remapping; legacy replay/frame/resume/ALSA 경로.
- 기존 scalar P1의 26개 regression tests. 공개 SDK에서 P2만 평가했으며 compiler success로 P1을 인증하지 않는다.
- P3 Camera/View/Format, gameplay interpretation, Forge, renderer, platform input mapping, native dynamic loader.
- trusted native callback panic/process termination, native sandbox/CPU·heap 제한, 외부 I/O rollback.
- 모든 가능한 save 구조의 exhaustive/fuzz 검증. 대표적인 malformed/canonical/accounting/reference/metadata 변경을 검사했다.
- plugin16/17, owner records256/257, scenes64/65, entities256/257 등 **모든 수치 상한의 각각의 exhaustion**; discovery 값 확인과 위에서 명시한 payload/command/event/frame/depth 경계를 실행했다.
- 여러 owner를 채워 정상 commit 단계에서 shared 8MiB save budget을 실제로 소진시키는 stress test. 8MiB 초과 restore input rejection은 실행했다.
- u64 identity/sequence/counter overflow의 모든 조합, error detail UTF-8 4096-byte 절단, dependency cycle·version·schema/action ID 중복의 모든 permutation.
- macOS/Windows/다른 CPU target; 이 결과는 exact Linux x86_64 rlib toolchain에 해당한다.

## 반환 파일

- `external_tests.rs`: 새로운 독립 Rust test suite (23 tests).
- `api_error_probe.rs`: 작은 public error-code 관찰 reproduction.
- `evidence/independent-final.log`: 정확한 runner command와 최종 PASS 출력.
- `evidence/independent-observations.log`: 실행 중 출력한 실제 discovery/limits/descriptors.
- `evidence/api-error-probe.log`, 예제/check/초기 실패 및 환경 로그.
- 원본 `MANIFEST.json` / `SHA256SUMS`, evidence 자체 `EVIDENCE_SHA256SUMS`.

**최종 판정: external P2 gate ACCEPTED (public Core SDK 범위); legacy 실행 UNVERIFIED.** 엔진/adapter 수정이나 P3 구현으로 결과를 보완하지 않았다.
