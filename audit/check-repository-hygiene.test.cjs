const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const cp=require('node:child_process');
const crypto=require('node:crypto');
const {audit,snapshot,fingerprint}=require('./check-repository-hygiene.cjs');
const work=path.resolve('work/hygiene-tests');
fs.mkdirSync(work,{recursive:true});
function repository() {
  const root=fs.mkdtempSync(path.join(work,'source with spaces '));
  const git=(...args)=>cp.execFileSync('git',args,{cwd:root,encoding:'utf8',stdio:['pipe','pipe','pipe']});
  const write=(name,value)=>{const file=path.join(root,name);fs.mkdirSync(path.dirname(file),{recursive:true});fs.writeFileSync(file,value);};
  git('init','--quiet');
  write('.gitignore','*.log\n');write('.gitattributes','*.log -text\n');
  write('audit/repository-hygiene-baseline.json',JSON.stringify({bundles:[]}));
  git('add','--','.gitignore','.gitattributes','audit/repository-hygiene-baseline.json');
  return {root,git,write,stage:(...names)=>git('add','-f','--',...names)};
}
function manifest(repo,bundle,names) {
  repo.write(`${bundle}/SHA256SUMS.json`,JSON.stringify({algorithm:'SHA-256',files:names.map(name=>{
    const bytes=fs.readFileSync(path.join(repo.root,bundle,name));
    return {path:name,bytes:bytes.length,sha256:crypto.createHash('sha256').update(bytes).digest('hex')};
  })}));
  repo.stage(`${bundle}/SHA256SUMS.json`);
}
test('an ignored local log cannot satisfy a staged evidence manifest',()=>{
  const r=repository(),b='evidence/phase-1/P1-07/fixture';
  r.write(`${b}/serial.log`,'m1.example\r\n');manifest(r,b,['serial.log']);
  assert.ok(audit(r.root).errors.some(e=>e.includes('missing from Git index')));
  r.stage(`${b}/serial.log`);
  assert.equal(audit(r.root).status,'passed');
  r.write(`${b}/serial.log`,'different\r\n');manifest(r,b,['serial.log']);
  assert.ok(audit(r.root).errors.some(e=>e.includes('differs from staged bytes')));
});
test('large legacy evidence is allowed only while its exact tracked snapshot survives',()=>{
  const r=repository(),b='evidence/phase-1/P1-06/fixture';
  r.write(`${b}/dump.log`,Buffer.alloc(300*1024,65));manifest(r,b,['dump.log']);r.stage(`${b}/dump.log`);
  const entries=snapshot(r.root).filter(e=>e.path.startsWith(b+'/'));
  r.write('audit/repository-hygiene-baseline.json',JSON.stringify({bundles:[{path:b,files:entries.length,index_fingerprint:fingerprint(entries)}]}));
  r.stage('audit/repository-hygiene-baseline.json');assert.equal(audit(r.root).status,'passed');
  r.write(`${b}/another.log`,'extra');r.stage(`${b}/another.log`);
  assert.ok(audit(r.root).errors.some(e=>e.includes('Historical evidence changed')));
});
test('new full tool dumps exceed the file and bundle budgets',()=>{
  const r=repository(),b='evidence/phase-1/P1-07/fixture';
  r.write(`${b}/full-dump.log`,Buffer.alloc(4*1024*1024+1,65));manifest(r,b,['full-dump.log']);r.stage(`${b}/full-dump.log`);
  const errors=audit(r.root).errors;
  assert.ok(errors.some(e=>e.includes('exceeds 256 KiB')));
  assert.ok(errors.some(e=>e.includes('exceeds 4 MiB')));
});
test('too many tiny reports still exceed the file-count budget',()=>{
  const r=repository(),b='evidence/phase-1/P1-07/fixture',names=[];
  for(let i=0;i<200;i++){const n=`${i}.json`;r.write(`${b}/${n}`,'{}');names.push(n);}
  manifest(r,b,names);r.stage(b);
  assert.ok(audit(r.root).errors.some(e=>e.includes('exceeds 200 files')));
});
test('a force-added stray log is rejected outside evidence',()=>{
  const r=repository();r.write('session.log','scratch');r.stage('session.log');
  assert.ok(audit(r.root).errors.some(e=>e.includes('Generated artifact tracked outside evidence')));
});
