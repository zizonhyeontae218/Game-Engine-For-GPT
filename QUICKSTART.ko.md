# PentaWorks v1.0 — 한국어 실행 안내

**현재 상태:** main = GE4G 0.3.0 Pentomino 알파 개발선. P0/P1과 에이전트 설정은 이미 병합됐다. `docs/pentomino/STATUS.md`부터 읽고 다음 작업을 시작한다.

## 적용

새 Codex 세션에서 최신 main 프로젝트 루트를 열고 아래 명령을 전달한다.
설정 파일을 다시 복사하거나 P0/P1을 반복하지 않는다.

```text
$pentomino-orchestrate를 사용해 GE4G 0.3.0 Pentomino 알파의 다음 Tiny Core 작업을 시작해.
docs/pentomino/STATUS.md와 P1_CONTRACT.md를 읽고, 완료된 P0/P1은 반복하지 마.
실제 저장소의 언어, 빌드 시스템, 테스트와 모듈 구성을 확인하기 위해
pentomino_scout를 읽기 전용으로 먼저 호출해.
내부 구현을 추측하지 말고, docs/pentomino/BRIEF.md에 맞춰
Tiny Core → View/Format → Gameplay 순으로 마일스톤을 정의해.
기능 구현 전 pentomino_architect에게 최소 공개 API 계약 초안을 만들게 하고,
pentomino_auditor에게 계약이 엔진 독립성을 깨지 않는지 검토시켜.
동시에 변경하는 파일 소유권은 겹치지 않도록 관리해.
현 단계에서 구현하지 않은 기능과 실행하지 않은 테스트는 UNVERIFIED로 표시하고,
작업 결과는 templates/RUNDOWN.md 형식으로 요약해.
```

## 작업자 10명

| 이름 | 담당 | 기본 권한 |
|---|---|---|
| pentomino_scout | 코드베이스 조사 | 읽기 전용 |
| pentomino_architect | 공개 계약·경계 설계 | 부모 설정 상속 |
| pentomino_core | Tiny Core 구현 | 부모 설정 상속 |
| pentomino_view | 카메라·표현 라이브러리 | 부모 설정 상속 |
| pentomino_plugin | 게임플레이 플러그인 | 부모 설정 상속 |
| pentomino_test | 통합·계약 테스트 작성 | 부모 설정 상속 |
| pentomino_auditor | 독립 아키텍처 감사 | 읽기 전용 |
| pentomino_consumer | 배포물 외부 사용자 테스트 | 읽기 전용 + 별도 작업 공간 필수 |
| pentomino_optimizer | 성능 회귀·최적화 | 부모 설정 상속 |
| pentomino_release | 알파·문서·Rundown | 부모 설정 상속 |

## 검증과 운영상 주의

`python3 scripts/validate_pentaworks.py`는 설정 문법·파일의 기본 유효성만 확인한다. `python3 -m unittest discover -s tests -v`는 패키징/게이트 검증 스크립트에 대한 테스트다. **Pentomino 기능 테스트가 아니다.**

외부 Consumer 실험은 공개 문서/배포 파일만 검토한 후 `consumer/ALLOWLIST.example.txt`를 실제 공개 파일 경로로 작성하고, 다음처럼 원본 저장소 **밖**의 새 디렉터리에 복사한다.

```bash
python3 scripts/stage_consumer_bundle.py \
  --root /path/to/GE4G \
  --allowlist /path/to/GE4G/consumer/ALLOWLIST.example.txt \
  --dest /tmp/ge4g-consumer-alpha1
```

그 후 **새 Codex 세션을 별도 작업 공간에서** 실행하고, 내부 엔진 저장소/비공개 도구 접근을 허용하지 않는다. 스크립트만으로 OS 수준 접근을 완전히 격리하는 것은 아니다.

## 범위 보호

P0-P1: Core. P2: Side/Vertical/Top-down/Classic 2D. P3: Turn → Realtime → Open world → Interaction/Platformer. P4: External Agent alpha. **스토리·에셋 Forge = v0.4 Mr. Smith.**

원본 콘셉트 문서는 `GE4G v0.3`으로 표기되어 있어 이 패키지에서도 그 명칭을 사용했다. 실제 저장소 이름을 확인한 후에만 변경한다.
