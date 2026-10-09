# 설치된 PentaWorks v1.0 설정

GE4G main에는 역할 10개와 스킬 2개가 이미 설치됐습니다. 파일을 다시 복사하거나
P0/P1을 반복하지 않습니다. [현재 상태](STATUS.md)와 [퀵스타트 명령](../../QUICKSTART.ko.md)을 사용하세요.

| 경로 | 용도 |
|---|---|
| `.codex/config.toml` | 프로젝트 역할 설정, 동시 작업자 최대 3명 |
| `.codex/agents/*.toml` | 작업자 10명, 모델을 고정하지 않고 부모 설정 상속 |
| `.agents/skills/pentomino-orchestrate/SKILL.md` | 작업 배분과 계약 검토 |
| `.agents/skills/pentomino-consumer-eval/SKILL.md` | 별도 워크스페이스의 공개 배포물 평가 |
| `AGENTS.md`, `docs/pentomino/` | 저장소 규칙, 구현 범위와 계약 |
| `templates/`, `scripts/validate_pentaworks.py` | 보고 형식과 설정 검증 |

새 Codex 세션에서 프로젝트 설정을 신뢰한 뒤 역할/스킬 발견을 확인합니다.
다른 저장소로 옮길 때는 기존 AGENTS.md·Codex 설정을 덮어쓰지 말고 병합합니다.
설정 검증은 `python3 scripts/validate_pentaworks.py`로 수행하며 엔진 기능 검증과 구분합니다.
Consumer는 공개 파일만 갖춘 별도 워크스페이스에서 실행합니다. 전체 소스 AgentKit은
엔진 개발용이므로 source-free Consumer 배포물로 사용하지 않습니다.
