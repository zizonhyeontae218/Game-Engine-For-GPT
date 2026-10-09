# Rundown — Pentomino P1 scalar lifecycle / 7c9dd6d
> 2026-10-09: 기존 체크리스트는 사용자 테스트 완료 보고를 반영해 완료로 표시했다. 아래 NOT RUN/UNVERIFIED/미구현은 작성 당시 기록이며 현재 기능 구현을 주장하지 않는다. 현재 main 범위는 docs/pentomino/STATUS.md 기준이다.


**TL;DR:** 독립 `ge4g-pentomino` 호스트에서 플러그인 등록·실패 복구·언로드·결정적 RNG·canonical save/atomic restore를 구현했다. 독립 공개 API 테스트26개 및 전체 Rust 테스트91개가 통과했고, auditor 재검토 기술 승인을 받았다. 기존 게임 실행 경로로 통합하거나0.3 제품을 배포한 것은 아니다.

**Status**
- [x] architect API/수치 limits/encoding freeze → auditor 검토 → 부모 보완 후 구현.
- [x] 독립 Tiny-host 크레이트, host-owned i64 상태·이벤트, 정확한 capability binding 및 non-cascading unload.
- [x] 독립 플러그인 A/B, 실패 주입·언로드·저장 변조·budget/overflow 테스트26개 PASS.
- [x] 부모 검토에서 pending 유실 restore 결함 발견·수정, 회귀 PASS. Auditor의 global retained-history 독립성 계약 공백 보완·경계 회귀 PASS.
- [x] 소스/테스트/공유 계약 배타 소유권, 순차 통합; 충돌 없음.
- [x] 전체 Tiny Core의 Scene/Entity·typed records/actions, View/Format, Gameplay: UNVERIFIED / 미구현.
- [x] 클라이언트 플랫폼·실기기·외부 source-free consumer: NOT RUN / UNVERIFIED.

**Changed:** 새 `crates/ge4g-pentomino/{Cargo.toml,src/lib.rs,tests/lifecycle_contract.rs}`, workspace member/lock 항목, `F(x).md`, `P1_CONTRACT.md`, plan/packets/audit/rundown, FILETREE 인덱스. 기존 core/project/runtime/render2d/client 소스와 schema/save/ABI1·제품 버전·Android 서명·지원 플랫폼 동작은 변경하지 않았다. 새 Rust API/저장은 experimental scalar subset이며 기존 CLI나 게임 import에서 실행되지 않는다.

**Evidence:** 구현 commit `7c9dd6d7e2c89984738fd7553e14b8cb6aa25107` (P0 baseline52e75c0), branch `pentomino/p1-lifecycle`. 아래 Rust 검사는 이 commit과 동일한 구현·테스트·manifest/lock 파일에서 실행했다. 마지막 통합은 기록/인덱스 문서 변경만 포함한다.

| Command / artifact | Observed result |
|---|---|
| `cargo tree --locked -p ge4g-pentomino --edges normal` | G1 scalar slice PASS: serde/json/sha2/thiserror 및 일반 유틸리티만; 기존 엔진/view/gameplay 의존성 없음, 독립 source review |
| `cargo test --locked -p ge4g-pentomino` |26 PASS /0 FAIL, G2/G6 scalar slice evidence |
| `cargo fmt --all --check` | PASS |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --locked --workspace` |91 PASS /0 FAIL,24 test/doc targets |
| `cargo run --locked -p ge4g-cli -- test examples/basement_demo --json` | PASS,160ticks/7 assertions; deterministic + save_reload + golden + PNG true |
| `cargo run --locked -p ge4g-cli -- test examples/flatland_pacman --json` | PASS,30ticks; deterministic + save_reload + golden + PNG true |
| `cargo run --locked -p ge4g-cli -- test examples/flatland_signal_yard --json` | PASS,714ticks; deterministic + save_reload + golden + PNG true |
| `cargo run --locked -p ge4g-cli -- test examples/flatland_harbor --json` | PASS,420ticks; deterministic + save_reload + golden + PNG true |
| `cargo clippy --locked -p ge4g-cli --no-default-features --all-targets -- -D warnings` | PASS |
| `cargo test --locked -p ge4g-cli --no-default-features` | PASS,9 tests |
| `python3 scripts/validate_pentaworks.py` | PASS, settings only |
| `python3 -m unittest discover -s tests -v` |5/5 PASS, tooling only |
| `python3 scripts/release_consistency.py --self-test` | PASS,19 current documents/release identity |
| `python3 scripts/filetree.py update` / `lint`, `git diff --check` | final integration results below |
| Flutter analyze/test/build, Windows embedded launch, Android build/device, view/composition/consumer/perf tests | NOT RUN / UNVERIFIED |

Reproducibility: Linux development host, Rust/Cargo1.99.0, locally installed rustfmt/clippy. Commands source `/home/agent/.cargo/env`; legacy audio crates use `PKG_CONFIG_PATH=/workspace/scratch/pentomino-build-deps/alsa/usr/lib/x86_64-linux-gnu/pkgconfig`. This directory contains locally extracted Debian ALSA1.2.14 development files; standard hosts can instead install libasound2-dev as existing CI does. System apt install failed due to host permissions; local SDK worked without repository patches. Execution logs are under `/workspace/scratch/pentomino-p1-*.log`; durable test sources and assertions are committed. No OS sandbox, performance uplift or physical acceptance claim.

The26 public cases include exact dependencies before callbacks, undeclared provider access, poisoned swallowed errors, whole-tick/init rollback including RNG/events, stale/cross-host/restored handles, invalid target ticks, strict save identity/encoding/version/bounds/accounting, complete pending replay, overflow, eviction/purge and fixed resource/command limits. Native AtomicBool is only a documented nonconforming failure-injection fixture. Real plugins must keep mutable state in host resources.

**Risks:** native plugins are trusted conforming code, not sandboxed. Hidden state/panic/process recovery remains UNVERIFIED. History retention is a global256-event budget; resources/RNG/current-tick output independence is tested, historical retention equality is not promised. Known Windows run37486665662/job112348341230 ZIP fixture digest failure remains unresolved; timestamp causality/fix UNVERIFIED and existing validation was not weakened. G3/G4/G5 and full-engine G1/G2/G6 remain UNVERIFIED/BLOCKED; these PASS results apply only to the additive scalar slice. No performance/release claim.

**Next branch:** next Tiny Core slice should establish the minimal format-facing records/selection contract and legacy compatibility adapter before P2 view integration. Then one removable Classic2D/Top-down adapter, remaining views, and Gameplay in BRIEF order. Current host lacks Scene/Entity/input actions and is not ready to claim named-view support. Forge0.4; no suspended client CI activation or automatic product release.

## Final integration log

2026-10-08: FILETREE update539 files, lint PASS(exit0), git diff --check PASS(exit0). Release-consistency self-test also PASS after final documentation updates. The evidence-log write is followed by a final index refresh/lint before the documentation commit; Rust sources/tests/manifests remain unchanged from7c9dd6d.
