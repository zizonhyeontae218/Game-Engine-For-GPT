import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.restore_flatland_release import download


class ReleaseRecoveryTests(unittest.TestCase):
    def test_only_expected_artifacts_are_written(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            expected = {"bytes": 3, "sha256": "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"}
            with self.assertRaises(ValueError):
                download("artifact", "https://example.com/private", expected, folder)
            with patch("urllib.request.urlopen", return_value=io.BytesIO(b"wrong")):
                with self.assertRaises(ValueError):
                    download("artifact", "https://files.oaiusercontent.com/public", expected, folder)
            self.assertFalse((folder / "artifact").exists())
            with patch("urllib.request.urlopen", return_value=io.BytesIO(b"abc")):
                download("artifact", "https://files.oaiusercontent.com/public", expected, folder)
            self.assertEqual((folder / "artifact").read_bytes(), b"abc")
