# 다음 작업 — Gameplay 첫 slice

P0–P3는 완료됐다. P2/P3 기존 테스트는 2026-10-09 사용자 보고로 모두 PASS 처리했다.
저장된 CI 로그와 사용자 외부 테스트 완료 근거는 구분한다. 정식 0.3 출시를 뜻하지 않는다.

## 첫 목표

**교체 가능한 턴 전투 플러그인 하나**를 구현한다. P2의 typed records/actions/events와
P3의 ReadFrame·View/Camera를 사용한다. Tiny Core에 HP·공격·턴 규칙을 넣지 않는다.
기존 FlatLand 전투 코드는 동작 참고 자료이며 Core에서 legacy gameplay를 호출하는 우회로로 쓰지 않는다.

1. 최신 main/CI를 확인하고 P2_CONTRACT.md와 P3_CONTRACT.md를 읽는다.
   architect가 최소 공개 계약을 작성하고 auditor가 의존성·저장·실패 경계를 검토한다.
2. 두 전투자의 공격/종료와 명시적 입력을 처리하는 최소 플러그인을 만든다.
   상태·RNG·이벤트는 host-owned 자료로 유지하고 View에는 화면 표현만 전달한다.
3. 헤드리스 리플레이 결과, 턴 중 저장/복원, 실패 시 원자적 복구, 제거/재설치,
   View 교체 시 전투 상태 보존을 검증한다. 공개 예제와 별도 Consumer 확인을 남긴다.

그 다음 realtime combat → open world → interaction/platformer 순서로 독립 플러그인을 늘린다.
Full3D renderer, 플랫폼 확대, Forge/스토리·자산 생성은 이 첫 slice에 포함하지 않는다.
Forge는 0.4다. 엔진 전체를 다시 쓰거나 완료된 P0–P3를 반복하지 않는다.

[현재 상태](STATUS.md) · [P2 계약](P2_CONTRACT.md) · [P3 계약](P3_CONTRACT.md)
