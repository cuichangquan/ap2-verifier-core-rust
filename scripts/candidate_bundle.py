"""Create a private local target bundle, never publish/upload."""
import hashlib,json,os,shutil,subprocess,sys,tarfile
from pathlib import Path
root=Path.cwd();out=Path(sys.argv[1]);triple=sys.argv[2];version='0.1.0-alpha.1'
name=f'ap2-verifier-core-{version}-{triple}';bundle=out/name
if bundle.exists():raise RuntimeError('Refusing to overwrite candidate bundle')
bundle.mkdir(parents=True)
# Use exactly the extracted source-package tree, excluding its build target.
for child in root.iterdir():
    if child.name=='target':continue
    if child.is_dir():shutil.copytree(child,bundle/child.name)
    else:shutil.copy2(child,bundle/child.name)
lib=bundle/'lib/libap2_verifier_core.so';lib.parent.mkdir();shutil.copy2(root/'target/release/libap2_verifier_core.so',lib)
shutil.copytree(out/'inventory',bundle/'inventory')
needed=subprocess.check_output(['readelf','-d',str(lib)],text=True)
versions=subprocess.check_output(['readelf','--version-info',str(lib)],text=True)
symbols=subprocess.check_output(['nm','-D','--defined-only',str(lib)],text=True)
if ' ap2_verify_v1\n' not in symbols:raise RuntimeError('Missing C ABI v1 symbol')
if '(RPATH)' in needed or '(RUNPATH)' in needed:raise RuntimeError('Unexpected embedded loader search path')
for file,text in [('dynamic.txt',needed),('symbol-versions.txt',versions),('exports.txt',symbols)]:
    (bundle/'inventory'/file).write_text(text)
metadata=dict(schema='ap2.local-native-candidate.1',version=version,target=triple,source_commit=os.environ['CANDIDATE_SOURCE_SHA'],
 source_crate_sha256=os.environ['CANDIDATE_CRATE_SHA'],rustc=subprocess.check_output(['rustc','-vV'],text=True),
 library=dict(path='lib/libap2_verifier_core.so',bytes=lib.stat().st_size,sha256=hashlib.sha256(lib.read_bytes()).hexdigest()),
 abi='ap2_verify_v1',request_contract='ap2-core-request.1',result_contract='ap2-core-result.1',publication='NOT_PUBLISHED')
(bundle/'candidate.json').write_text(json.dumps(metadata,indent=2)+'\n')
archive=out/(name+'.tar.gz')
with tarfile.open(archive,'w:gz') as t:t.add(bundle,arcname=name)
print(json.dumps(dict(archive=archive.name,bytes=archive.stat().st_size,sha256=hashlib.sha256(archive.read_bytes()).hexdigest(),library=metadata['library'])))
