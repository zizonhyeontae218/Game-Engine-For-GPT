# 다음 작업 — P5 Turn Combat

P0–P3는 main에 병합됐고 기존 테스트는 2026-10-09 사용자 보고로 PASS 처리됐다.
P4 작업 브랜치에는 최소 Gameplay·네이티브 연결이 구현돼 실제 Rust/C 호출과
CPU 프레임을 검증했다. 현재 병합·CI·외부 인증 상태는 P4_RUNDOWN.md에서 확인한다.
내부 검증 성공은 정식 0.3 출시나 Android/Windows 실행물 완성을 뜻하지 않는다.

## P4에서 이어갈 실제 경계

- 영속 StateProvider와 제거 가능한 TurnCombat은 공개 P2 API만 사용한다.
  입력 한 번은 결정 Event Tick과 적용 Action Tick의 두 단계다.
- NativeBridge가 실제 CoreHost/ViewHost/CameraFrame/CPU 프레임을 연결한다.
  저장2와 Presentation 저장1은 독립 문자열이며 세션 복원은 전체 staging 후 교체한다.
- 공개 관측은 구조화된 Map 키를 JSON 객체로 쓰지 않고 정렬된 DTO 배열을 쓴다.
- Gameplay 제거는 실제 CoreHost::remove이고, 전투 권위 상태·Scene/Entity는
  StateProvider에 남는다. ABI1·FlatLand 게임/저장 호환성은 유지한다.
- 독립 새 세션/source-free Consumer 인증은 내부 packaged consumer 재현과
  구분한다. P4 SDK와 새 Consumer 테스트로 외부 근거를 남긴다.

## 순서

1. P4_CONTRACT/P4_HANDOFF와 최신 main/PR 상태를 확인한다. P0–P3를 반복하지 않는다.
2. P5에서 더 풍부한 턴 액션·효과·비용·결과 규칙을 작은 JSON 기반 플러그인으로
   확장한다. 전투 규칙을 Tiny Core에 넣지 않으며 변경된 계약과 저장 migration을
   architect/auditor가 검토한다. 원자성·결정론·권위 상태 보존 회귀를 유지한다.
3. 네이티브 경계가 안정되면 기존 Flutter 클라이언트를 재사용해 Android 최소
   실행 레이어와 게임·실행기·런타임이 내장된 Windows Playable Alpha를 검증한다.
   실제 조작·저장/복원·종료, 빌드/서명/체크섬과 플랫폼별 근거가 필요하다.

Realtime combat, open world, interaction/platformer는 이후 독립 플러그인 작업이다.
Full3D renderer·플랫폼 확대·Forge/스토리·자산 생성은 P4/P5 첫 경계에 포함하지 않는다.
Forge는 0.4다.

[현재 상태](STATUS.md) · [P4 계약](P4_CONTRACT.md) · [P4 검증](P4_RUNDOWN.md) · [P5 Handoff](P4_HANDOFF.md)
