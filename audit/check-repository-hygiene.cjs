// Audit the Git index, so ignored local artifacts cannot satisfy committed evidence.
const cp = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const LIMITS = { bundleBytes: 4 * 1024 * 1024, bundleFiles: 200, fileBytes: 256 * 1024 };
function git(root, args, input) {
  return cp.execFileSync('git', args, {cwd:root,input,encoding:'utf8',maxBuffer:16*1024*1024});
}
function snapshot(root) {
  const entries = git(root,['ls-files','--stage','-z']).split('\0').filter(Boolean).map(row => {
    const m = /^(\d+) ([0-9a-f]+) (\d)\t([\s\S]+)$/.exec(row);
    if (!m || m[3] !== '0') throw new Error('Unmerged or malformed Git index');
    return {path:m[4],blob:m[2]};
  });
  const sizes = new Map(git(root,['cat-file','--batch-check=%(objectname) %(objectsize)'],entries.map(e=>e.blob).join('\n')+'\n').trim().split('\n').map(row=>{
    const [blob,size]=row.split(' ');return [blob,Number(size)];
  }));
  return entries.map(e=>({...e,bytes:sizes.get(e.blob)}));
}
function fingerprint(entries) {
  return crypto.createHash('sha256').update([...entries].sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0).map(e=>`${e.blob} ${e.path}\n`).join('')).digest('hex');
}
// Read each distinct staged blob once, in bounded batches. Thousands of separate
// `git show` processes made an unchanged index audit expensive on Windows.
function readBlobs(root, entries, consume) {
  const unique=[...new Map(entries.map(e=>[e.blob,e])).values()];
  let batch=[],bytes=0;
  function flush() {
    if(!batch.length) return;
    const raw=cp.execFileSync('git',['cat-file','--batch'],{
      cwd:root,input:batch.map(e=>e.blob).join('\n')+'\n',maxBuffer:Math.max(16*1024*1024,bytes)
    });
    let offset=0;
    for(const entry of batch) {
      const end=raw.indexOf(10,offset);
      if(end<offset) throw new Error('Missing staged blob header');
      const header=raw.subarray(offset,end).toString('ascii').split(' ');
      if(header.length!==3 || header[0]!==entry.blob || header[1]!=='blob' ||
         !/^\d+$/.test(header[2]) || Number(header[2])!==entry.bytes) throw new Error('Staged blob identity/size mismatch');
      offset=end+1;
      const content=raw.subarray(offset,offset+entry.bytes);
      if(content.length!==entry.bytes || raw[offset+entry.bytes]!==10) throw new Error('Truncated staged blob');
      consume(entry,content);
      offset+=entry.bytes+1;
    }
    if(offset!==raw.length) throw new Error('Unexpected staged blob output');
    batch=[];bytes=0;
  }
  for(const entry of unique) {
    if(entry.bytes>16*1024*1024) throw new Error('Staged evidence blob exceeds audit read bound');
    if(bytes+entry.bytes+128>8*1024*1024) flush();
    batch.push(entry);bytes+=entry.bytes+128;
  }
  flush();
}
function bundleOf(name) {
  const parts=name.split('/');
  return parts[1]==='phase-0' ? 'evidence/phase-0' : parts.slice(0,Math.min(4,parts.length-1)).join('/');
}
// This exception is limited to the owner-authorized migration of frozen bundles.
// Their remaining index fingerprints prevent editing a manifest to hide new data.
function localOnlyHistorical(name, baseline) {
  return !!baseline.untracking && name.startsWith('evidence/') &&
    (name.endsWith('.log') || name.includes('/source/')) &&
    baseline.bundles.some(b=>b.path===bundleOf(name) && b.local_only_files>0);
}
function audit(root) {
  const files=snapshot(root), tracked=new Set(files.map(e=>e.path)), errors=[];
  const baseline=JSON.parse(git(root,['show',':audit/repository-hygiene-baseline.json']));
  const groups=new Map();
  for(const file of files) {
    if(file.path==='evidence/README.md') {
      if(file.bytes>LIMITS.fileBytes) errors.push('Evidence availability notice exceeds 256 KiB');
    } else if(file.path.startsWith('evidence/')) {
      const bundle=bundleOf(file.path);
      if(!groups.has(bundle)) groups.set(bundle,[]);
      groups.get(bundle).push(file);
    } else if(/(^|\/)(work|build|dist|target|node_modules|__pycache__|\.cache)\//.test(file.path) || /\.(log|elf|iso|o|obj|pdb|ilk|rlib|rmeta|qcow2|vhdx?|vmdk|fd|dmp|pyc|tmp|bak|swp)$/i.test(file.path)) {
      errors.push(`Generated artifact tracked outside evidence: ${file.path}`);
    }
  }
  const bundles=[];
  for(const [bundle,entries] of groups) {
    const bytes=entries.reduce((s,e)=>s+e.bytes,0), old=baseline.bundles.find(b=>b.path===bundle);
    if(old) {
      if(entries.length!==old.files || fingerprint(entries)!==old.index_fingerprint) errors.push(`Historical evidence changed: ${bundle}; retain its immutable snapshot`);
    } else {
      if(bytes>LIMITS.bundleBytes) errors.push(`Evidence bundle exceeds 4 MiB: ${bundle}`);
      if(entries.length>LIMITS.bundleFiles) errors.push(`Evidence bundle exceeds 200 files: ${bundle}`);
      for(const file of entries) if(file.bytes>LIMITS.fileBytes) errors.push(`Evidence file exceeds 256 KiB: ${file.path}`);
      const manifest=`${bundle}/SHA256SUMS.json`;
      if(!tracked.has(manifest)) errors.push(`New evidence bundle needs a tracked SHA256SUMS.json: ${bundle}`);
    }
    bundles.push({path:bundle,files:entries.length,bytes,historical:!!old});
  }
  for(const old of baseline.bundles) if(!groups.has(old.path)) errors.push(`Historical evidence removed: ${old.path}`);
  const manifests=files.filter(e=>e.path.startsWith('evidence/') && path.posix.basename(e.path)==='SHA256SUMS.json');
  const parsed=new Map(),byPath=new Map(files.map(e=>[e.path,e])),checks=[];
  readBlobs(root,manifests,(entry,bytes)=>parsed.set(entry.blob,JSON.parse(bytes.toString('utf8'))));
  for(const file of manifests) {
    const manifest=parsed.get(file.blob);
    const parent=path.posix.dirname(file.path);
    for(const entry of manifest.files) {
      const target=path.posix.normalize(path.posix.join(parent,entry.path));
      if(!target.startsWith(parent+'/') || !tracked.has(target)) {
        if(!target.startsWith(parent+'/') || !localOnlyHistorical(target,baseline)) errors.push(`Frozen evidence member missing from Git index: ${target}`);
      } else if(!baseline.bundles.some(b=>b.path===bundleOf(file.path))) {
        checks.push({file:byPath.get(target),expected:entry,target});
      }
    }
  }
  const observed=new Map();
  readBlobs(root,checks.map(c=>c.file),(entry,bytes)=>observed.set(entry.blob,{
    sha256:crypto.createHash('sha256').update(bytes).digest('hex'),bytes:bytes.length
  }));
  for(const check of checks) {
    const actual=observed.get(check.file.blob),entry=check.expected;
    if(actual.sha256!==entry.sha256 || (entry.bytes!==undefined && actual.bytes!==entry.bytes)) errors.push(`Frozen evidence checksum differs from staged bytes: ${check.target}`);
  }
  return {status:errors.length?'failed':'passed',scope:'Git index; historical metadata protected, declared historical logs/source copies local-only, new evidence bounded',limits:LIMITS,trackedFiles:files.length,trackedBytes:files.reduce((s,e)=>s+e.bytes,0),logFiles:files.filter(e=>e.path.endsWith('.log')).length,logBytes:files.filter(e=>e.path.endsWith('.log')).reduce((s,e)=>s+e.bytes,0),bundles,errors};
}
if(require.main===module) {
  const root=process.cwd(),report=audit(root);
  fs.mkdirSync(path.join(root,'work'),{recursive:true});
  fs.writeFileSync(path.join(root,'work/repository-hygiene-results.json'),JSON.stringify(report,null,2)+'\n');
  if(process.argv.includes('--quiet')) {
    if(report.errors.length) for(const error of report.errors) console.error(error);
    else console.log('Repository hygiene passed.');
  } else console.log(JSON.stringify(report,null,2));
  process.exitCode=report.errors.length?1:0;
}
module.exports={audit,snapshot,fingerprint,bundleOf,localOnlyHistorical,readBlobs};
