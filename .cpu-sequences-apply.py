import base64
import gzip
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

BASE = '7cdfb50617dd40c05146be6042a61a64d7fe3c72'
PAYLOAD_SHA = '7b37927944ece999e670b662850e26620087b57c0b4c2f0f9b8c8e0fc6b5f0b3'
transport = Path('.cpu-sequences.payload.gz').read_bytes()
assert hashlib.sha1(f'blob {len(transport)}\0'.encode() + transport).hexdigest() == '4355f7e8273acf47bba0e68180213fbbc78aa015'
# Restore three characters lost in tool transport, never an unchecked source edit.
encoded = base64.b64encode(transport).decode()
assert encoded.count('G6c93S5E+RsQ') == 1
encoded = encoded.replace('G6c93S5E+RsQ', 'G6c93S5E+XU+RsQ') + '='
compressed = base64.b64decode(encoded, validate=True)
assert hashlib.sha256(compressed).hexdigest() == '268c400468d2a51edfef3b53b43f22f366ce9b28fbe47e8ee0bdb0e77c14131c'
raw = gzip.decompress(compressed)
assert hashlib.sha256(raw).hexdigest() == PAYLOAD_SHA
items = json.loads(raw)
assert len(items) == 18
paths, manifest = [], []
for item in items:
    path = Path(item['path'])
    assert not path.is_absolute() and '..' not in path.parts
    assert path.parts[0] in {'README.md', 'docs', 'tools'} and str(path) not in paths
    if item['before_sha256'] is None:
        assert not path.exists(), path
        before = ''
    else:
        data = path.read_bytes()
        assert hashlib.sha256(data).hexdigest() == item['before_sha256'], path
        before = data.decode('utf-8')
    after = before
    previous = len(before)
    for start, end, replacement in reversed(item['edits']):
        assert 0 <= start <= end <= previous
        after = after[:start] + replacement + after[end:]
        previous = start
    data = after.encode('utf-8')
    assert hashlib.sha256(data).hexdigest() == item['after_sha256'], path
    assert hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest() == item['git_blob_sha'], path
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    paths.append(str(path))
    manifest.append({'path': str(path), 'sha': item['git_blob_sha'], 'sha256': item['after_sha256'], 'size': len(data)})
# Source guards must see the actual production workflow, not this diagnostic one.
Path('.github/workflows/build.yml').write_bytes(subprocess.check_output(['git', 'show', BASE + ':.github/workflows/build.yml']))
subprocess.run(['git', 'add', '-N', '--', *paths], check=True)
subprocess.run(['git', 'diff', '--check', '--', *paths], check=True)
out = Path('tmp/cpu-sequences')
out.mkdir(parents=True, exist_ok=True)
(out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
(out / 'review.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary', BASE, '--', *paths]))
(out / 'revision.txt').write_bytes(subprocess.check_output(['git', 'rev-parse', 'HEAD']))
with tarfile.open(out / 'source.tar.gz', 'w:gz') as archive:
    for path in paths:
        archive.add(path, arcname=path)
print('Verified 18 source files against exact original and final content hashes.')
