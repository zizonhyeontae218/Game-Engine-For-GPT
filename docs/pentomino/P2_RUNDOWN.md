# Rundown — GE4G v0.3 Pentomino P2 / 48d7f4a

**TL;DR:** P2 Tiny Core와 별도 legacy snapshot adapter를 구현했고 workspace116개
테스트 및 네 게임 regression이 통과했다. 사용자의
별도 GPT Work 공개 Core SDK **23/23 PASS / Gate ACCEPTED**를 반영해 P2를 완료했다.
외부 레거시 실행은 UNVERIFIED이며, main 병합·정식 릴리스·P3 구현은 별도 작업이다.

Source artifact: `48d7f4aa6932c9031afee722e0c58f7756b864c8`,
GE4G `0.3.0-alpha.1`, branch `pentomino/p2-tiny-core`.
Baseline main: `2d1ffd060389109f341fb5854ce07890f445f3f9`.
FlatLand maintenance: `293ba513f5727d4a7a0a59476c9a98eb193aaa25` (unchanged).
보고서와 evidence 추가는 기능 commit 이후의 문서 변경이다.

**Status — 구현 상태 체크리스트**

- [x] 플러그인별 독립 history256, commit/pending128, 명시적 dropped/retired/purged accounting.
- [x] Scene/Entity identity와 lifetime, stale/foreign handles, unload cleanup과 dependency-safe refs.
- [x] bool/i64/UTF-8/bytes/Scene·Entity refs/bounded lists의 선언적 schema/records.
- [x] next-tick bool/bounded-i64 InputFrame, action privacy와 canonical replay/save continuation.
- [x] records/objects/identity/RNG/events/input/tick 전체 원자적 실패 복구, save2/strict restore/discovery.
- [x] 별도 legacy adapter의 World→typed projection/actions, 실제 Core install과 ref remapping.
- [x] P1 26개 테스트 및 원래 root 구현 보존, schema/save/ABI1·release/0.2 유지.
- [x] 내부 검증·독립 감사·공개 예제·source-free ZIP과 업로드 후 checksum 확인.
- [x] 외부 GPT Work 공개 Core SDK: **23/23 PASS / Gate ACCEPTED** — 사용자 보고서.
- [ ] 외부 레거시 실행: **UNVERIFIED** — 보고서 승인 범위에서 제외.
- [ ] Camera/View/Format/Gameplay: **미구현, UNVERIFIED — P3 이후**.

**Changed — 변경된 public contracts**

기존 root scalar `Host/Plugin/ErrorCode`, contract1/save1은 그대로다. 변경은
`pub mod p2;` 추가뿐이다. P1의 전역 history256 동작도 호환성 namespace에
유지하므로 **독립 retention은 새 `p2::CoreHost`를 선택해야 한다**. Save1을
save2로 조용히 변환하지 않고 VersionMismatch로 거부한다.

새 `CorePlugin/CoreDescriptor`, `CoreHost/CoreContext/CoreTransaction`,
`SceneRef/EntityRef`와 opaque handles, `SchemaId/RecordSchema/FieldType/Value`,
`ActionId/InputFrame`, `Selection/ReadFrame/OwnerHistory/CoreDiscovery`를 제공한다.
contract2.0.0/save2; record64KiB/save8MiB와 선언적 bounds를 capped writer로
검사한다. Stable ref는 저장된 timeline 데이터이며 runtime handle은 freshness를
담당한다. Restore는 saved allocator를 정확히 복구하고 모든 runtime handles를
갱신한다. ReadFrame은 Rust typed DTO이며 nonempty structured-key map의 직접
JSON 표현은 정의하지 않는다. Portable canonical wire는 Host::save() bytes다.

`ge4g-pentomino-legacy`가 legacy Project/World/State를 일반 typed records와
Scene/Entity에 투영한다. World는 Core callback 밖에서 기존 실행 권위를 유지하며
candidate 실행→projection validation→commit 순서로 갱신한다. Imported plugin은
**immutable snapshot**이고 Core tick이 legacy gameplay를 계속 실행하지 않는다.
카메라·presentation·legacy save/resume은 기존 runtime 영역에 남긴다. Source-known
Replay buttons는 bridge가 검증하고, plugin factory는 trusted caller의 supplied
hash 형식/Bool(true)를 검증한다; hash provenance를 역산한다고 주장하지 않는다.

정확한 공개 계약: [P2_CONTRACT.md](P2_CONTRACT.md).
[공개 SDK 안내](../public/pentomino-p2/README.md),
[public API](../public/pentomino-p2/PUBLIC_API.md),
[plugin/discovery](../public/pentomino-p2/PLUGIN_DISCOVERY.md).

**Evidence — 테스트 결과**

아래 모든 PASS는 위 source commit의 실제 실행 결과다.
[Exact commands + observed results](evidence/p2/verification.json), 동일 폴더의
원본 logs, [audit](P2_AUDIT.md)와 [package evidence](evidence/p2/package.json)를
함께 보존한다. Rust/Cargo1.99.0; Linux workspace의 ALSA SDK는 공식 Debian
libasound2-dev1.2.14를 로컬로 추출하여 PKG_CONFIG_PATH에 제공했다.

| 실제 실행 command | Observed result |
|---|---|
| `cargo fmt --all --check` | PASS; diagnostics 없음 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --locked --workspace` | **116 PASS**,0 failed; P1 26 + P2 21 + adapter4 포함 |
| `cargo clippy --locked -p ge4g-cli --no-default-features --all-targets -- -D warnings` | PASS |
| `cargo test --locked -p ge4g-cli --no-default-features` | **9 PASS**,0 failed |
| `cargo run --locked -p ge4g-pentomino --example p2_public` | PASS;301 tick,B retained256/dropped44,canonical continuation hash 동일 |
| `python3 scripts/check_p2_boundaries.py` | PASS; Core transitive engine deps 없음, P1 hash/root·legacy paths 보존 |
| `python3 scripts/validate_pentaworks.py` | PASS |
| `python3 -m unittest discover -s tests` | PASS5 |
| `python3 scripts/release_consistency.py --self-test` | PASS;21 current docs/version checks |
| `python3 scripts/package_p2_external.py --out /workspace/scratch/ge4g-p2-public-48d7f4a` | PASS; source-free compiler-pinned ZIP |
| `python3 scripts/filetree.py update` / `python3 scripts/filetree.py lint` | PASS;586-file navigation/hash registry |
| unpacked SDK: `python3 run_public.py --check` / `python3 run_public.py` | PASS; checksums/compiler와 예제 컴파일·301 tick 실행; **내부 패키지 점검** |

P2 독립 테스트는 retention isolation/save/remove/accounting/overflow, 객체 수명과
cross-owner/stale/cross-host refs, plugin unload, schema/type/depth/UTF-8/bytes/list
bounds, canonical64KiB 및 aggregate8MiB 초과, action privacy/ordering/replay,
commands4096/4097·객체·records·actions/event quota, suppressed-error poison,
late foreign reference collision, failed initialize/tick의 RNG/events/input/identity
전체 rollback, malformed save atomic rejection과 restore continuation을 검증한다.
[실행 로그](evidence/p2/test-workspace.log)의 실제 test names를 참조한다.

네 게임 모두 `cargo run --locked -p ge4g-cli -- test examples/<name> --json` 실행:

| Legacy example | ticks / assertions | deterministic / save_reload / golden_checked / png_capture |
|---|---|---|
| `basement_demo` |160 /7| 모두 true, PASS |
| `flatland_pacman` |30 /1| 모두 true, PASS |
| `flatland_signal_yard` |714 /1| 모두 true, PASS |
| `flatland_harbor` |420 /1| 모두 true, PASS |

Adapter 독립4개 테스트는 실제 네 게임 typed projection/Core install/select,
Unicode scene/state, replay mapping, World 직접 실행과 bridge 결과 동일성,
oversized post-tick projection/legacy errors의 atomic rollback, forged input/
projection rejection과 save equality를 검증한다. P1 테스트 checksum:
`ce87260ccd1c7c5fe98daac774d70db90f7c6aed981c6c1767e953a288e8461a` (변경 없음).

**발견하고 수정한 결함**

1. P1의 공유 history로 A가 B 과거를 evict하는 문제를 P2의 독립 owner budgets로 해결.
   P1 호환 계약/테스트를 삭제하거나 약화하지 않았다.
2. 계약 감사의 suffix/accounting 누락과 serialization 전 한도 검사 요구를 설계 동결
   전에 반영했다. Stable ref/restore generation 및 Replay tick mapping도 명확히 했다.
3. 실제 실패 재현: owner별 정상인 저장에서 A tick1/seq1, B tick2/seq0로 바꾸면
   restore가 수락했다. Global sequence로 합친 retained events의 tick 비감소 검사를
   추가한 뒤 동일 독립 regression이 PASS했다. [Fixed log](evidence/p2/chronology-fixed.log).
4. 공개 예제의 중첩 pattern clippy 경고를 수정해 전체 target strict lint를 통과시켰다.
   잘못된 adapter test fixture는 Project에 state 선언을 추가하여 정합성을 복구했고,
   rejection assertions는 유지했다.

**Risks — 알려진 위험 / UNVERIFIED**

- **범위:** 외부 public Core Gate는 ACCEPTED다. Codex가 외부 테스트를 수행한 것은
  아니다. 외부 legacy/game 실행·exhaustive/fuzz·모든 수치 quota 조합은 승인 범위 밖이다.
  보고서가 나열한 신규 test source/raw logs는 이번 세션에 첨부되지 않았다.
- **제한:** snapshot adapter의 continuous synchronization/legacy save migration,
  Core 경유 CLI/client 실행은 미구현. 기존 regression의 성공은 새 실행 경로의 성공이 아니다.
- **제한:** per-owner eviction은 격리하지만 shared8MiB save cap 소진은 전체 commit을
  거부한다. Callback 외부 효과/hidden state/panic/process recovery와 악성 native
  sandbox는 UNVERIFIED. 제공 플러그인은 trusted deterministic linked code다.
- **제한:** SDK rlib는 정확한 compiler와 Linux x86_64 target에 고정된다.
  다른 platform/compiler SDK 및 외부 source-free legacy 실행은 UNVERIFIED.
- **CI:** b05c082의 GitHub acceptance·Windows·Android jobs는 모두 SUCCESS로 확인했다.
  실기기·최종 서명·배포 검증은 UNVERIFIED.
  알려진 Windows ZIP fixture timestamp/digest intermittent failure의 원인/수정도
  UNVERIFIED로 보존한다. 지원 플랫폼 확대나 기존 바이너리 교체는 하지 않았다.

**Next branch — P3로 넘길 항목**

[P3_HANDOFF.md](P3_HANDOFF.md): **Camera & View/Format 집중 개발이 다음 단계**다.
외부 공개 Core23/23 보고서를 반영했고 blocking 기능 결함은 없다. P3 공개 계약은
scout→architect→auditor로 설계한다. Camera lifetime/save, coordinate transforms, replaceable
Classic2D/Side/Vertical Scroll/Top-down과 backend/asset capabilities는 Core 밖의
플러그인/어댑터에서 다룬다. P2에 P3 카메라/renderer/Gameplay를 선행 구현하지 않았다.
Forge/generation/dynamic native loading도 구현하지 않았다.

**외부 GPT Work 테스트 결과 — 사용자 보고서 반영**

[사용자가 반환한 원본 보고서](evidence/p2/external/P2_External_Test_Report.ko.md),
[acceptance provenance](evidence/p2/external/acceptance.json),
[검증 전 PR CI readback](evidence/p2/external/ci-before-acceptance.json).

별도 source-free 세션에서 신규23개 테스트와 공개 예제/checksum/compiler 검사를
실행했고6개 공개 SDK challenge가 PASS했다. 최종 판정은 **ACCEPTED — public Core SDK**.
레거시 adapter/게임, 기존 scalar P1, P3 및 미실행 경계는 외부 인증하지 않았다.

실제 외부 명령: `python3 run_public.py --check`, `python3 run_public.py`,
`python3 run_public.py --source external_tests.rs --test`, `./consumer-output --nocapture`,
`python3 run_public.py --source api_error_probe.rs`. 관찰:23 passed/0 failed.
Tested commit48d7f4aa6932c9031afee722e0c58f7756b864c8; Core rlib SHA256
`bd0b89a7855ccd5cf6ad8b5839002cbe3f79c38c875e4559e70eba9fe2c8bd35`는 전달본과 일치한다.

원본 [배포 ZIP](https://drive.google.com/file/d/1vC9BBFuC3Hl_7miMpUqlMrF3WpyBTlTn/view)
SHA256은 `717c27399d794b8a2a8615070a06ae3a19ae6c640492b76215f0f940ae89e6d0`.
외부 report의 테스트 ZIP SHA256은
`da0c744914c910479d49a5dbe017526e8366736cede1931515f2e2b851e9c76d`이며,
사용자가 **재압축**했다고 확인했다. 두 archive 해시를 동일하다고 주장하지 않는다.
보고서의 manifest30개와 원본32개 차이는 원본 report 그대로 기록한다; 새 manifest는
미수신이다. Core 동일성은 정확한 rlib hash로 검증했다. 원래 ZIP/runner의
external_gate UNVERIFIED는 패키지 생성 시점의 immutable 기록으로 유지한다.

Gate-blocking 결함은 없었다. 소비자 피드백으로 declared field bound InvalidRecord와
canonical/transaction quota BudgetExceeded, input frame InvalidInput, unknown selected
owner StaleHandle을 명시한 공개 오류 표를 추가했다. Contract의 proposed/runtime
UNVERIFIED 표기를 구현/검증 상태로 갱신했고, 향후 runner는 rustc 부재를 traceback
대신 BLOCKED로 안내한다. Core/adapter Rust 구현·save bytes·버전은 바꾸지 않았다.
문서/runner 후속 점검은 내부 실행이며 외부23개 테스트의 재실행으로 주장하지 않는다.

후속 내부 검증: [followup-checks.json](evidence/p2/external/followup-checks.json)에
missing-rustc BLOCKED 출력, exact compiler/checksum PASS, release consistency,
Pentaworks/tooling/dependency PASS command/result를 기록했다. FILETREE update/lint와
문서 링크 검사도 통과했다. Core/adapter implementation diff 없음.
