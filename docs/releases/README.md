# 릴리즈와 배포

**안정판:** [v0.2.0 FlatLand](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/tag/v0.2.0).
**개발선:** main의 GE4G 0.3.0 Pentomino `0.3.0-alpha.1`. 현재 정식 0.3 배포는 없습니다.

## 0.2.0 배포 파일

2026-10-09: 비공개 Drive의 원본 14개 파일을 복구하고 모두 기존 크기·SHA256과 대조했습니다.
GitHub Actions에서 공개 릴리즈 첨부를 복구합니다. 실제 첨부 여부는 릴리즈 Assets에서 확인하세요.
태그·원본 APK 서명·실행 파일 바이트는 유지하며 C SDK와 AgentKit을 추가합니다.

| 파일 | 용도 |
|---|---|
| `GE4G-FlatLand-0.2.0-Android.apk` | 서명된 최소 실행기. 설치 후 `.ge4g` 가져오기 |
| `GE4G-Harbor-0.2.0-Windows.zip` | 바람항 공방·Flutter 실행기·Rust 런타임 내장. 압축 해제 후 `ge4g_client.exe` 실행 |
| `FlatLand-Harbor-0.2.0.ge4g` | 실행기에 가져오는 data-only 게임 |
| `FlatLand-Harbor-0.2.0-Source.zip` | 수정 가능한 샘플 게임·CC0 자산·저작 예제 |
| `GE4G-0.2.0-C-SDK-Windows.zip` | ABI1 헤더·원본 DLL·C 예제·컴파일된 콘솔·샘플·라이선스·출처 |
| `GE4G-0.2.0-AgentKit.zip` | v0.2.0 엔진 소스·공개 문서·샘플, `START-HERE.md`부터 시작 |
| `GE4G-0.2.0-SHA256SUMS.txt` | 원본과 추가 패키지의 SHA256 |
| 라이선스·검증 JSON·Windows Evidence ZIP | 자산 출처와 당시 테스트/렌더링 증거 |

Windows는 ZIP 안의 DLL·data 디렉터리를 함께 유지합니다. Android 앱 ID는
`dev.ge4g.ge4g_client`, 0.2.0은 versionCode8이며 기존 고정 인증서를 사용합니다.
원본 `SHA256SUMS.txt`와 `README-ko.txt`도 역사 기록으로 보존합니다.
추가 SDK의 C 예제 검증은 별도 복구 워크플로의 결과이며 원래 0.2 테스트와 구분합니다.

## 버전과 공개 기준

- 공개 다운로드는 GitHub Releases의 영구 첨부 파일을 사용합니다. 비공개 Drive 접근을 요구하지 않습니다.
- Actions 아티팩트는 임시 CI 결과입니다. 서명되지 않은 APK를 설치용으로 안내하지 않습니다.
- `release/0.2`는 안정판 유지보수, `main`은 0.3 알파입니다. 알파 릴리즈는 별도 태그·prerelease로 구분합니다.
- 안정판 AgentKit과 main의 Pentomino 개발 역할을 섞지 않습니다. 0.3 개발은 main을 복제하고
  [퀵스타트](../../QUICKSTART.ko.md)를 사용합니다. 전체 소스 AgentKit은 source-free Consumer SDK가 아닙니다.
- 공개 릴리즈에는 실행기·샘플·C SDK·개발 자료·라이선스·체크섬과 버전/commit 근거를 포함합니다.
- 과거 Drive 파일과 서명 백업은 그대로 보존하고 공유 권한을 변경하지 않습니다.

[0.2 출시·서명·검증 기록](../FLATLAND_RELEASE.md) · [과거 RC 기록](history/README.md)

## 원본 복구 재실행

`Restore FlatLand release assets` 워크플로는 원본 14개에 한정해 임시 다운로드 URL을 받습니다.
저장된 manifest의 파일명·크기·해시를 검사한 뒤 기존 공개 v0.2.0에 첨부합니다.
URL은 Git에 저장하거나 로그에 출력하지 않습니다. Windows SDK는 실제 배포 DLL로 C 예제를
컴파일하고 validate/open을 확인한 후에만 업로드합니다. 개인 키·Drive 인증 토큰은 사용하지 않습니다.
