# Rundown — Pentomino P0 / baseline 293ba513

**TL;DR:** 실제 main 저장소를 읽기 전용으로 조사하고 Tiny Core → View/Format → Gameplay 계획과 최소 공개 API 제안을 작성했다. 독립 감사의 계약 보완 3건 및 이력 규칙을 반영하고 재검토 기술 승인을 받았다. 엔진 기능 구현 완료를 의미하지 않는다.

**Status**
- [x] P0-S: Rust2024 Cargo 7개 crate / Flutter0.2.0+8 / 실제 API·의존성·테스트 경로 조사 (`P0_SCOUT.md`).
- [x] P0-A: 제안 공개 API, data-only import, bounded lifecycle, view/genre 독립 경계 (`P0_API_PROPOSAL.md`).
- [x] P0-R: 최초 CHANGES_REQUIRED → 부모 순차 보완 → auditor 재검토 기술 승인 (`P0_AUDIT.md`).
- [x] TASK_PACKET 형식의 배타적 소유권 및 active ExecPlan. 동시 파일 소유권 충돌 없음.
- [x] 사용자 첨부 원본 QUICKSTART 복구와 설정/Python 검사 통과.
- [ ] P1 기능·G1/G2/G6 실행 증거: UNVERIFIED. 수치 limits/encoding 동결 필요.
- [ ] View/Format·Gameplay·플랫폼·외부 consumer 검증: UNVERIFIED; G4/G5는 선행 구현/공개 alpha가 없어 BLOCKED.

**Changed:** `docs/exec-plans/active/pentomino-p0{,-packets}.md`, `docs/pentomino/P0_{SCOUT,API_PROPOSAL,AUDIT,RUNDOWN}.md`, 첨부 원본 `QUICKSTART.ko.md`, 생성 FILETREE/해시 인덱스. 공개 API는 PROPOSAL / UNVERIFIED이고 기존 소스·schema/save/ABI1·버전·서명·클라이언트 지원 동작은 변경하지 않았다.

**Evidence:** 기준 main HEAD `293ba513f5727d4a7a0a59476c9a98eb193aaa25`, 초기 clean; 작업 branch `pentomino/p0-contract`.

| 명령 / 증거 | 관찰 결과 |
|---|---|
| `git rev-parse HEAD`, `git status --short`, `git remote -v` | 실제 main 기준·초기 clean·요청 origin 확인 |
| `python3 scripts/validate_pentaworks.py` | 최초 missing QUICKSTART FAIL; 원본 복구 후 PASS |
| `python3 -m unittest discover -s tests -v` | 최초 같은 누락으로 1/5 FAIL; 복구 후 5/5 PASS |
| `python3 scripts/release_consistency.py --self-test` | PASS: 19 current documents/release identity |
| `cmp QUICKSTART.ko.md <user attachment>` | exit0, 원본과 byte-identical |
| `python3 scripts/filetree.py update` 및 `lint` | 최종 갱신/검사 결과는 아래 integration log |
| `git diff --check` | 최종 결과는 아래 integration log |
| Cargo fmt/clippy/test/replay 및 Flutter analyze/test/build | NOT RUN / UNVERIFIED: PATH에 Rust/Flutter 도구 없음 |
| 역할 실행 | scout→architect→auditor 및 auditor 재검토 실제 수행; OS 수준 read-only 강제는 UNVERIFIED |

설정 검사는 엔진 작동, 플러그인 언로드, 뷰 교체, 물리 기기, 소비자 격리의 증거가 아니다. G0 inventory만 source-backed이며 G1–G7 runtime PASS를 주장하지 않는다. G8은 문서/변경 범위 검토상 Forge 기능 추가 없음.

**Risks:** MEDIUM: wire encoding·수치 budgets·history accounting의 P1 동결 및 실행 검증 필요. 기존 core의 camera/texture/FlatLand 필드와 runtime→Project→gameplay 결합은 확인된 추출 위험이다. 기존 Windows run37486665662/job112348341230의 ZIP digest 실패를 보존하며 timestamp 원인/수정은 UNVERIFIED. 기존 save/digest 검증을 약화하지 않는다.

**Next branch:** P1-C — 수치 limits와 encoding을 정한 뒤 최소 host + 독립 dummy plugin 2개, G1/G2/G6 테스트. 이후 교체 가능한 View/Format → Turn/Realtime/Open-world/Interaction/Platformer. Android/Windows 유지; 다른 플랫폼 자동 활성화 없음. Forge는0.4, 외부 consumer는 별도 source-free 세션 및 공개 alpha 이후.

## Integration log

2026-10-08: final configuration validation PASS, Python tests 5/5 PASS, release consistency self-test PASS. FILETREE update recorded 531 files; lint and git diff --check exited0. The evidence log update requires one final index refresh/check before committing. No runtime or platform tests executed.
