# C ABI 1 최소 예제

이 예제는 기존 FlatLand 네이티브 런타임을 호출합니다. Pentomino P1 scalar 호스트 API와는
별개입니다. 엔진 소스 수정이나 JSON 파서 의존성 없이 한 줄씩 요청하고 응답을 출력합니다.
반환 문자열은 엔진의 `ge4g_free_string`으로 해제합니다.

## Windows x64

Visual Studio의 **x64 Native Tools Command Prompt**에서 SDK 압축을 풀고 실행합니다.
SDK의 `ge4g_client.dll`은 검증된 0.2.0 Windows 배포물에서 그대로 가져옵니다.
배포 파일 구성은 [릴리즈 안내](../../docs/releases/README.md)를 참고하세요.

```bat
lib /nologo /def:ge4g_client.def /machine:x64 /out:ge4g_client.lib
cl /nologo /W4 /Iinclude main.c ge4g_client.lib /Fe:ge4g-c.exe
ge4g-c.exe
```

SDK가 없으면 저장소 루트에서 `cargo build --locked --release -p ge4g-client` 후
`target/release/ge4g_client.dll`과 헤더를 사용합니다. 이때 main에서 빌드한 라이브러리는
0.3 알파의 보존된 FlatLand 런타임이며 0.2 배포 DLL과 버전이 다릅니다.

```json
{"op":"validate","project":"game"}
{"op":"open","project":"game"}
```

`open` 응답의 `session` 값을 다음 요청에 사용하세요. 아래 `1`은 예시입니다.

```json
{"op":"advance","session":1,"ticks":1}
{"op":"observe","session":1,"compact":true}
{"op":"close","session":1}
```

Ctrl+Z, Enter로 입력을 마칩니다. 각 요청은 한 줄, 최대 4094바이트입니다.
`ok:false`와 `error`는 엔진 응답이므로 호출자가 확인해야 합니다. 이 콘솔은 프레임을
표시하지 않습니다. 화면 출력은 `ge4g_frame_copy`를 호출하는 별도 어댑터가 맡습니다.

## Linux 개발 환경

```sh
cargo build --locked --release -p ge4g-client
cc -std=c11 -Wall -Wextra -Werror -Icrates/ge4g-client/include examples/c_abi/main.c \
  -Ltarget/release -lge4g_client -Wl,-rpath,'$ORIGIN' -o target/release/ge4g-c
printf '%s\n' '{"op":"validate","project":"examples/basement_demo"}' | target/release/ge4g-c
```

Linux는 이 예제의 개발 검증 환경입니다. 공개 실행기 지원 범위는 Android·Windows입니다.
