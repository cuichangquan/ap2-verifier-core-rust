"""Offline conservative SPDX inventory + verbatim license texts for local bundles.
Run inside the official Rust image after cargo metadata --locked --offline.
This includes the full resolved graph, not a claim every package is linked.
"""
import collections,datetime,hashlib,json,os,shutil,subprocess,sys,tomllib,urllib.parse
from pathlib import Path
root=Path.cwd();out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--offline','--format-version','1']))
lock=tomllib.loads((root/'Cargo.lock').read_text())
checksums={(p['name'],p['version']):p.get('checksum') for p in lock['package']}
ids={p['id']:'SPDXRef-'+hashlib.sha256((p['name']+'@'+p['version']).encode()).hexdigest()[:16] for p in metadata['packages']}
packages=[];summaries=[];missing=[]
for p in sorted(metadata['packages'],key=lambda p:p['name']):
    license=p.get('license') or 'NOASSERTION'
    directory=Path(p['manifest_path']).parent
    files=set()
    for entry in directory.iterdir():
        if entry.name.upper().startswith(('LICENSE','COPYING','NOTICE','UNLICENSE')):
            if entry.is_file():files.add(entry)
            elif entry.is_dir():files.update(f for f in entry.rglob('*') if f.is_file())
    if p.get('license_file'):
        f=(directory/p['license_file']).resolve()
        if not f.is_relative_to(directory.resolve()):raise RuntimeError('License path escapes package')
        if f.is_file():files.add(f)
    if not files:missing.append(p['name'])
    dest=out/'LICENSES'/(p['name']+'-'+p['version']);dest.mkdir(parents=True,exist_ok=True)
    record=[]
    for f in sorted(files):
        relative=f.relative_to(directory);target=dest/relative;target.parent.mkdir(parents=True,exist_ok=True)
        data=f.read_bytes();target.write_bytes(data)
        record.append(dict(file=relative.as_posix(),sha256=hashlib.sha256(data).hexdigest(),bytes=len(data)))
    spdx_license=license.replace('MIT/Apache-2.0','MIT OR Apache-2.0')
    item=dict(SPDXID=ids[p['id']],name=p['name'],versionInfo=p['version'],downloadLocation='NOASSERTION',filesAnalyzed=False,
              licenseDeclared=spdx_license,licenseConcluded='NOASSERTION',copyrightText='NOASSERTION',
              licenseComments='Cargo metadata declared: '+license)
    checksum=checksums.get((p['name'],p['version']))
    if checksum:
        item['checksums']=[dict(algorithm='SHA256',checksumValue=checksum)]
        item['downloadLocation']=f'https://crates.io/api/v1/crates/{p["name"]}/{p["version"]}/download'
        item['externalRefs']=[dict(referenceCategory='PACKAGE-MANAGER',referenceType='purl',referenceLocator=f'pkg:cargo/{p["name"]}@{urllib.parse.quote(p["version"],safe="")}')]
    packages.append(item);summaries.append(dict(name=p['name'],version=p['version'],declared_license=license,source_archive_sha256=checksum,license_files=record))
if missing:raise RuntimeError('Missing original license texts: '+','.join(missing))
relationships=[dict(spdxElementId='SPDXRef-DOCUMENT',relationshipType='DESCRIBES',relatedSpdxElement=ids[metadata['resolve']['root']])]
for n in metadata['resolve']['nodes']:
    relationships.extend(dict(spdxElementId=ids[n['id']],relationshipType='DEPENDS_ON',relatedSpdxElement=ids[d]) for d in n['dependencies'])
epoch=int(os.environ.get('SOURCE_DATE_EPOCH','0'))
created=datetime.datetime.fromtimestamp(epoch,datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
namespace=hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest()
sbom=dict(spdxVersion='SPDX-2.3',dataLicense='CC0-1.0',SPDXID='SPDXRef-DOCUMENT',name='ap2-verifier-core-candidate-resolved-graph',
 documentNamespace='https://spdx.org/spdxdocs/ap2-core-'+namespace,
 creationInfo=dict(creators=['Tool: ap2-candidate-inventory-1'],created=created,creatorComment='Resolved Cargo graph superset including build/other-platform dependencies; toolchain/system libraries recorded separately.'),
 packages=packages,relationships=relationships)
(out/'sbom.spdx.json').write_text(json.dumps(sbom,indent=2)+'\n')
(out/'dependency-licenses.json').write_text(json.dumps(summaries,indent=2)+'\n')
# Include verbatim Rust standard-library notices, as the native library links std.
sysroot=Path(subprocess.check_output(['rustc','--print','sysroot'],text=True).strip())
doc=sysroot/'share/doc/rust';td=out/'LICENSES/rust-toolchain';td.mkdir(parents=True,exist_ok=True)
shutil.copytree(doc/'licenses',td/'licenses',dirs_exist_ok=True)
shutil.copy2(doc/'COPYRIGHT-library.html',td/'COPYRIGHT-library.html')
print(json.dumps(dict(resolved_packages=len(packages),dependency_packages=len(packages)-1,licenses=dict(collections.Counter(p['declared_license'] for p in summaries)),missing_license_texts=missing)))
