# P2 외부 테스트 Quickstart

별도의 GPT Work 세션에 ZIP만 전달하세요. GE4G 내부 구현 저장소나 내부 테스트를
함께 제공하지 마세요. P2는 외부 검증을 기다리는 완료 후보입니다.

1. 새 작업 디렉터리에 ZIP을 풀고 `MANIFEST.json`의 compiler/target을 확인합니다.
   현재 SDK는 Linux x86_64용이며 정확히 같은 Rust compiler가 필요합니다.
2. 다음 명령으로 checksum, 공개 예제 컴파일과 실행을 검증합니다.

```sh
python3 run_public.py --check
python3 run_public.py
```

예제는 A/B 두 플러그인, 선언적 typed schema, Scene/Entity, input, RNG와 events를
사용합니다. 300 tick 후 B는 256 events를 보존하고 dropped=44입니다. A 제거 후
B selection이 동일해야 하고, save/restore 후 runtime handle은 stale이어야 합니다.
301번째 tick의 replay와 save가 같으면 JSON `ok:true`를 출력합니다.

공개 API로 직접 작성한 `my_plugin.rs` 또는 독립 Rust test 파일도 실행할 수 있습니다.

```sh
python3 run_public.py --source my_plugin.rs
python3 run_public.py --source my_tests.rs --test
```

외부 tester는 `EXTERNAL_TESTER_TASK.md`의 명령서를 따라 새로운 테스트를 작성하고,
실행 command, SDK commit/checksum, 실제 출력과 판정을 보고하세요. API를 읽기만
했거나 실행하지 못한 항목은 `UNVERIFIED` 또는 `BLOCKED`로 표시합니다.

이 패키지는 기존 게임 실행용 런처가 아닙니다. 레거시 실행 경로의 내부 regression
증거는 별도 Rundown에 기록됩니다. 실제 외부 GPT Work 테스트는 사용자가 수행하며,
Codex의 패키지 컴파일 확인을 외부 검증으로 취급하면 안 됩니다.
