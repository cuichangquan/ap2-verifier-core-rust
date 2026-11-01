"""Fetch immutable official AP2 source in the build stage; never at runtime."""
import hashlib,io,tarfile,urllib.request
from pathlib import Path
COMMIT='b4587ac1d055888a73b4b21750973cffba961793'
EXPECTED='526288baea429019db3f4a7ccb8aad15d629b8c760db3f30ec511d32a53b0d47'
url=f'https://codeload.github.com/google-agentic-commerce/AP2/tar.gz/{COMMIT}'
with urllib.request.urlopen(url,timeout=60) as response:
    data=response.read(4*1024*1024+1)
if len(data)>4*1024*1024 or hashlib.sha256(data).hexdigest()!=EXPECTED:
    raise RuntimeError('Official source archive hash/size mismatch')
with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz') as archive:
    archive.extractall('/tmp/ap2-source',filter='data')
Path('/tmp/ap2-source',f'AP2-{COMMIT}').rename('/official')
print(f'official AP2 {COMMIT}; archive sha256={EXPECTED}; bytes={len(data)}')
