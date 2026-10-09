# GE4G 문서

main은 **0.3.0 Pentomino 알파**, 안정판은 **0.2.0 FlatLand**입니다.
아래 문서가 설명하는 개발선과 실제 구현 범위를 확인하고 필요한 진입점만 읽으세요.

## 사용자와 게임 작성자

- [배포 파일·설치·검증](releases/README.md)
- [Android·Windows 실행기](CLIENT.md)
- [FlatLand 저작 빠른 시작](FLATLAND_QUICKSTART.md) · [상세 저작 계약](FLATLAND_AUTHORING.md)
- [바람항 공방 샘플](../examples/flatland_harbor/README.md)
- [C ABI 예제](../examples/c_abi/README.md) · [CLI 계약](CLI_CONTRACT.md)

## Pentomino 개발자와 에이전트

- [퀵스타트 명령](../QUICKSTART.ko.md) · [저장소 작업 규칙](../AGENTS.md)
- [현재 main 상태](pentomino/STATUS.md) · [인수인계](pentomino/HANDOFF.md)
- [다음 Gameplay 작업](pentomino/NEXT.md) · [범위](pentomino/BRIEF.md)
- [P1 계약](pentomino/P1_CONTRACT.md) · [P2 계약](pentomino/P2_CONTRACT.md) · [P3 계약](pentomino/P3_CONTRACT.md)
- [아키텍처](ARCHITECTURE.md) · [검증 기준](TESTPLAN.md)
- [역할 설정 안내](pentomino/SETUP.md) · [파일 지도](../FILETREE.md)

## 릴리즈와 역사

- [릴리즈 구성·버전 정책](releases/README.md) · [변경 기록](RELEASE_NOTES.md)
- [0.2.0 출시 기록](FLATLAND_RELEASE.md)
- [과거 RC 기록과 배포 증거](releases/history/README.md)
- [완료된 작업 계획](exec-plans/completed/)

과거 Rundown의 NOT RUN/UNVERIFIED는 작성 당시 기록입니다. 기존 테스트 체크리스트는
2026-10-09 사용자 완료 보고를 반영했습니다. 현재 구현 범위는 STATUS.md가 기준입니다.
