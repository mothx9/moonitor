#!/usr/bin/env python3
"""Record source and artifact hashes without including private configuration."""
import hashlib, json, pathlib, subprocess, sys
root=pathlib.Path(__file__).resolve().parents[1]
def run(*args):return subprocess.check_output(args,cwd=root,text=True).strip()
paths=run('git','ls-files','-co','--exclude-standard').splitlines()
source={p:hashlib.sha256((root/p).read_bytes()).hexdigest() for p in sorted(set(paths)) if (root/p).is_file()}
manifest={'commit':run('git','rev-parse','HEAD'),'dirty':bool(run('git','status','--porcelain')),'rustc':run('rustc','-Vv'),'source_sha256':hashlib.sha256(json.dumps(source,sort_keys=True).encode()).hexdigest(),'files':source,'binaries':{}}
for name in ['moonitor','moonitor-agent']:
    p=root/'target/release'/name
    if p.exists():manifest['binaries'][name]=hashlib.sha256(p.read_bytes()).hexdigest()
path=pathlib.Path(sys.argv[1]);path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(manifest,indent=2)+'\n')
