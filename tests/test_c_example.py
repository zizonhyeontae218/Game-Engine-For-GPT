import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipUnless(shutil.which("cc"), "C compiler not installed")
class CExampleTests(unittest.TestCase):
    def test_requests_memory_ownership_and_rejection(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            stub = folder / "stub.c"
            stub.write_text('''#include <stdlib.h>
#include <string.h>
#include "ge4g_client.h"
static int outstanding;
uint32_t ge4g_abi_version(void) { return 1; }
char *ge4g_request_json(const char *s) {
    if (outstanding || strstr(s, "null")) return NULL;
    char *r = malloc(strlen(s) + 1);
    if (r) { strcpy(r, s); r[strcspn(r, "\\n")] = 0; outstanding = 1; }
    return r;
}
void ge4g_free_string(char *s) { free(s); outstanding = 0; }
''')
            executable = folder / "example"
            subprocess.run(["cc", "-std=c11", "-Wall", "-Wextra", "-Werror",
                            "-I" + str(ROOT / "crates/ge4g-client/include"),
                            str(ROOT / "examples/c_abi/main.c"), str(stub),
                            "-o", str(executable)], check=True)
            def run(request):
                return subprocess.run([str(executable)], input=request,
                                      text=True, capture_output=True)
            result = run('{"op":"open"}\n{"op":"close"}')
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stdout.splitlines(), ['{"op":"open"}', '{"op":"close"}'])
            self.assertEqual(run("x" * 5000 + "\n").returncode, 1)
            self.assertEqual(run("null\n").returncode, 1)
