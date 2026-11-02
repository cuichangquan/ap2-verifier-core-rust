"""Explicit maintenance command, not part of routine validation.
Resolve only exact versions at official PyPI; lock every wheel hash to support
both Linux architectures without permitting source builds or unpinned fallback.
"""
import hashlib,json,urllib.request
from pathlib import Path
root=Path('/work')
lines=['# Exact versions and official PyPI wheel SHA256; no sdists permitted.']
for spec in (root/'requirements-oracle.txt').read_text().splitlines():
    name,version=spec.split('==')
    with urllib.request.urlopen(f'https://pypi.org/pypi/{name}/{version}/json',timeout=30) as response:
        metadata=json.load(response)
    hashes=sorted({f['digests']['sha256'] for f in metadata['urls'] if f['packagetype']=='bdist_wheel' and not f.get('yanked',False)})
    if not hashes:raise RuntimeError(f'No official wheels for {spec}')
    lines.append(spec+' \\\n'+''.join('    --hash=sha256:'+h+(' \\\n' if i<len(hashes)-1 else '\n') for i,h in enumerate(hashes)))
(root/'requirements-oracle.lock').write_text('\n'.join(lines)+'\n')
print(json.dumps(dict(packages=len(lines)-1,lock_sha256=hashlib.sha256((root/'requirements-oracle.lock').read_bytes()).hexdigest())))
