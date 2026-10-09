# P3 외부 SDK 빠른 시작

저장소나 내부 구현을 읽지 않는 새 GPT Work 세션에서 ZIP을 풀어 사용한다.
MANIFEST.json의 정확한 Rust compiler/target이 필요하다. 이 SDK는 Linux-hosted
Rust 개발 검증용이며 새 Linux 클라이언트 배포가 아니다.

ZIP_LZMA 압축을 사용한다. Python 표준 라이브러리로 압축을 푼다.

```sh
python3 -m zipfile -e GE4G-P3-camera-public.zip p3-sdk
cd p3-sdk
python3 run_public.py --check
python3 run_public.py
python3 run_public.py --example fifth_demo
python3 run_public.py --source my_camera_tests.rs --test
```

기본 example 실행만으로 외부 Gate를 PASS 처리하지 않는다. 별도 consumer
테스트를 작성하고 실행 결과를 기록한다. fifth_demo example은 패키지에
포함된 기존 다섯 번째 콘텐츠 fixture를 읽으며 새 게임을 만들지 않는다.
