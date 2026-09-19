import base64
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile

BASE = '09d097fd070ceb0a633d161c0c34d7185530abee'
EXPECTED = '3b05d98ab556f0faed71f24fd6762881a7e58e9d5013a6cdc82a8be34e8e6c63'
raw = gzip.decompress(base64.b64decode(''.join(Path(f'.cpu-final-part-{i}.b64').read_text().strip() for i in range(5)), validate=True))
assert hashlib.sha256(raw).hexdigest() == EXPECTED, 'transport digest mismatch'
edits = json.loads(raw)
assert len(edits) == 18
paths = []
manifest = []
for item in edits:
    path = Path(item['path'])
    assert not path.is_absolute() and '..' not in path.parts and path.parts[0] in {'README.md', 'docs', 'tools'}
    assert str(path) not in paths
    if item['before_sha256'] is None:
        assert not path.exists(), f'new file already exists: {path}'
        old = ''
    else:
        data = path.read_bytes()
        assert hashlib.sha256(data).hexdigest() == item['before_sha256'], f'base changed: {path}'
        old = data.decode('utf-8')
    result = old
    end_limit = len(old)
    for start, end, replacement in reversed(item['edits']):
        assert 0 <= start <= end <= end_limit
        result = result[:start] + replacement + result[end:]
        end_limit = start
    data = result.encode('utf-8')
    assert hashlib.sha256(data).hexdigest() == item['after_sha256'], f'result digest mismatch: {path}'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    paths.append(str(path))
    manifest.append({'path': str(path), 'sha256': item['after_sha256'], 'size': len(data), 'git_blob_sha': hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()})
# Tests inspect the production workflow. The diagnostic workflow is not a proposed source change.
workflow = subprocess.check_output(['git', 'show', f'{BASE}:.github/workflows/build.yml'])
Path('.github/workflows/build.yml').write_bytes(workflow)
subprocess.run(['git', 'add', '-N', '--', *paths], check=True)
subprocess.run(['git', 'diff', '--check', '--', *paths], check=True)
out = Path('tmp/cpu-final')
out.mkdir(parents=True, exist_ok=True)
(out / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
(out / 'payload.sha256').write_text(EXPECTED + '\n')
(out / 'revision.txt').write_bytes(subprocess.check_output(['git', 'rev-parse', 'HEAD']))
(out / 'review.patch').write_bytes(subprocess.check_output(['git', 'diff', '--binary', BASE, '--', *paths]))
with tarfile.open(out / 'source.tar.gz', 'w:gz') as archive:
    for path in paths:
        archive.add(path, arcname=path)
print(f'Applied and verified {len(paths)} source files; diagnostic files are not part of the patch.')
