"""Summarize exact local artifacts after both candidate validations finish."""
import hashlib,json,re,shutil,tarfile,sys,tomllib
from pathlib import Path
root=Path('/work');sha=sys.argv[1];base=root/'.cache/candidate'/sha;out=root/'docs/candidate'
records=[];crate_hashes=[]
for arch,triple in [('arm64','aarch64-unknown-linux-gnu'),('amd64','x86_64-unknown-linux-gnu')]:
    directory=base/arch
    crate=directory/'ap2-verifier-core-0.1.0-alpha.1.crate'
    native=directory/f'ap2-verifier-core-0.1.0-alpha.1-{triple}.tar.gz'
    bundle=directory/f'ap2-verifier-core-0.1.0-alpha.1-{triple}'
    assert crate.is_file() and native.is_file() and bundle.is_dir()
    with tarfile.open(crate) as t:
        names=t.getnames()
        manifest=tomllib.loads(t.extractfile('ap2-verifier-core-0.1.0-alpha.1/Cargo.toml').read().decode())
        assert manifest['package']['publish'] in (False,[]), 'Publication gate relaxed'
        assert manifest['package']['version']=='0.1.0-alpha.1'
        assert all(not any(bad in name for bad in ['/target/','/.cache/','/docs/candidate/','/scripts/']) for name in names)
        required=['src/lib.rs','include/ap2_verifier.h','adapters/ruby/lib/ap2_verifier_core.rb','LICENSE','THIRD_PARTY_NOTICES.md','PACKAGE_README.md','conformance/v1/manifest.json']
        assert all('ap2-verifier-core-0.1.0-alpha.1/'+name in names for name in required)
        assert not any(name.endswith('ruby_baseline.rb') for name in names)
        assert b'RUSTSEC-2026-0387' in t.extractfile('ap2-verifier-core-0.1.0-alpha.1/PACKAGE_README.md').read()
    content=[]
    for file in [crate,native,bundle/'lib/libap2_verifier_core.so']:
        b=file.read_bytes();content.append(dict(path=str(file.relative_to(root)),bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
    crate_hashes.append(content[0]['sha256'])
    info=json.loads((bundle/'candidate.json').read_text())
    dyn=(bundle/'inventory/dynamic.txt').read_text();versions=(bundle/'inventory/symbol-versions.txt').read_text()
    records.append(dict(target=triple,execution='native Docker' if arch=='arm64' else 'Docker emulation on Apple Silicon',
      source_package_files=names,artifacts=content,library=info['library'],rustc=info['rustc'],abi=info['abi'],
      needed_libraries=re.findall(r'Shared library: \[([^]]+)\]',dyn),required_symbol_versions=sorted(set(re.findall(r'(?:GLIBC|GCC)_[0-9.]+',versions))),
      ruby=['3.4.12','4.0.9'],functional_package_tests='PASS',release_status='HOLD_RUSTSEC-2026-0387'))
assert crate_hashes[0]==crate_hashes[1], 'Source archives differ across target packaging'
for name in ['sbom.spdx.json','dependency-licenses.json']:
    shutil.copy2(base/'arm64/inventory'/name,out/name)
manifest=dict(schema='ap2.unpublished-candidate-materials.1',version='0.1.0-alpha.1',source_commit=sha,
 publication='NOT_PUBLISHED',release_status='HOLD_RUSTSEC-2026-0387',source_archive_same_bytes_both_targets=True,
 rustsec_db='2295466c4e9df26647fd67d8fa1c705ac2fc7f81',cargo_audit='0.22.2',advisory_exit_code=1,targets=records)
(out/'artifacts.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(json.dumps(dict(source=sha,source_crate_sha256=crate_hashes[0],targets=[r['target'] for r in records],release_status=manifest['release_status'])))
