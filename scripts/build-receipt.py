#!/usr/bin/env python3
"""Record signed bundle hashes outside the signature's own resource envelope."""
import hashlib
import json
import pathlib
import sys

app = pathlib.Path(sys.argv[1])
output = pathlib.Path(sys.argv[2])
root = pathlib.Path(__file__).resolve().parents[1]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

manifest_path = app / 'Contents/Resources/build-manifest.json'
manifest = json.loads(manifest_path.read_text())
unsigned = digest(root / 'target/release/moonitor')
assert unsigned == manifest['binaries']['moonitor'], 'Build changed after manifest creation'
receipt = {
    'commit': manifest['commit'],
    'dirty': manifest['dirty'],
    'source_sha256': manifest['source_sha256'],
    'build_binary_sha256': unsigned,
    'signed_bundle_binary_sha256': digest(app / 'Contents/MacOS/moonitor'),
    'manifest_sha256': digest(manifest_path),
    'icon_sha256': digest(app / 'Contents/Resources/moonitor.icns'),
    'note': 'Signing changes Mach-O bytes. Build and signed hashes intentionally differ.',
}
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(receipt, indent=2) + '\n')
