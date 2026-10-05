#!/usr/bin/env python3
"""Sign an unsigned Android candidate with the preserved, pinned release certificate."""
import argparse
import hashlib
from pathlib import Path
import shutil
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def certificate_sha256(apk):
    data = Path(apk).read_bytes()
    eocd = data.rfind(b"PK\x05\x06")
    if eocd < 0:
        raise ValueError("APK has no ZIP end record")
    central = struct.unpack_from("<I", data, eocd + 16)[0]
    if data[central - 16:central] != b"APK Sig Block 42":
        raise ValueError("APK has no Android v2/v3 signing block")
    size = struct.unpack_from("<Q", data, central - 24)[0]
    cursor = central - size

    def part(value, offset=0):
        length = struct.unpack_from("<I", value, offset)[0]
        return value[offset + 4:offset + 4 + length], offset + 4 + length

    while cursor < central - 24:
        length = struct.unpack_from("<Q", data, cursor)[0]
        cursor += 8
        ident = struct.unpack_from("<I", data, cursor)[0]
        value = data[cursor + 4:cursor + length]
        cursor += length
        if ident == 0x7109871A:
            signers, _ = part(value)
            signer, _ = part(signers)
            signed, _ = part(signer)
            _, offset = part(signed)
            certificates, _ = part(signed, offset)
            certificate, _ = part(certificates)
            return hashlib.sha256(certificate).hexdigest()
    raise ValueError("APK has no v2 signer certificate")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("apk", type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--keystore", required=True, type=Path)
    parser.add_argument("--password-file", required=True, type=Path)
    parser.add_argument("--previous-apk", type=Path)
    parser.add_argument("--apksigner-jar", type=Path)
    args = parser.parse_args()
    expected = (ROOT / "client/android/signing-certificate.sha256").read_text().strip()
    if args.previous_apk and certificate_sha256(args.previous_apk) != expected:
        raise ValueError("Previous APK certificate differs; this would require uninstalling")
    if args.out.exists():
        raise ValueError("Refusing to overwrite an existing signed APK")
    signer = (["java", "-jar", str(args.apksigner_jar.resolve())] if args.apksigner_jar
              else [shutil.which("apksigner") or "apksigner"])
    args.out.parent.mkdir(parents=True, exist_ok=True)
    try:
        subprocess.run(signer + ["sign", "--ks", str(args.keystore.resolve()),
                       "--ks-key-alias", "ge4g", "--ks-pass", "file:" + str(args.password_file.resolve()),
                       "--key-pass", "file:" + str(args.password_file.resolve()),
                       "--out", str(args.out), str(args.apk)], check=True)
        subprocess.run(signer + ["verify", "--verbose", "--print-certs", str(args.out)], check=True)
        if certificate_sha256(args.out) != expected:
            raise ValueError("Release certificate differs from pinned SHA256")
    except Exception:
        args.out.unlink(missing_ok=True)
        raise
    print("Verified stable Android release certificate:", expected)


if __name__ == "__main__":
    main()
