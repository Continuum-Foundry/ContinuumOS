const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve('.');
function walk(p) { return fs.readdirSync(p,{withFileTypes:true}).flatMap(d=>d.isDirectory()?walk(path.join(p,d.name)):[path.join(p,d.name)]); }
const docs = walk(path.join(root,'docs'));
const md = [...docs.filter(p=>p.endsWith('.md')), path.join(root,'AGENTS.md'), path.join(root,'CLAUDE.md'), path.join(root,'README.md')];
const json = docs.filter(p=>p.endsWith('.json'));
const errors = [];
const texts = new Map(md.map(p=>[p,fs.readFileSync(p,'utf8')]));
function slug(s) { return s.toLowerCase().replace(/\[([^\]]+)\]\([^)]*\)/g,'$1').replace(/[`*_]/g,'').replace(/[^\p{L}\p{N}\s-]/gu,'').trim().replace(/\s/g,'-'); }
let links=0;
for (const [file,raw] of texts) {
  const text=raw.replace(/```[\s\S]*?```/g,'');
  for (const match of text.matchAll(/\[[^\]\n]*\]\(([^)\n]+)\)/g)) {
    let ref=match[1].replace(/^<|>$/g,'');
    if (/^(https?:|mailto:|urn:)/i.test(ref)) continue;
    links++;
    let [name,anchor] = ref.split('#');
    const dest=name?path.resolve(path.dirname(file),decodeURIComponent(name)):file;
    if (!fs.existsSync(dest)) { errors.push(`${path.relative(root,file)} missing ${ref}`); continue; }
    if (anchor && dest.endsWith('.md')) {
      const dt=fs.readFileSync(dest,'utf8');
      const anchors=[...dt.matchAll(/^#{1,6}\s+(.+)$/gm)].map(m=>slug(m[1]));
      if (!anchors.includes(anchor)) errors.push(`${path.relative(root,file)} missing anchor ${ref}`);
    }
  }
  if (/turn\d+(?:search|view)|cite/.test(raw)) errors.push(`Internal browser citation leaked in ${file}`);
}
for(const file of json) { try { JSON.parse(fs.readFileSync(file,'utf8')); } catch(e) {errors.push(`Invalid JSON ${file}: ${e.message}`);} }
const sourceText=fs.readFileSync(path.join(root,'docs/01-research/sources.md'),'utf8');
const sourceIds=[...sourceText.matchAll(/^### (S\d+)$/gm)].map(m=>m[1]);
if(new Set(sourceIds).size!==sourceIds.length) errors.push('Duplicate source IDs');
const req=fs.readFileSync(path.join(root,'docs/00-overview/requirements.md'),'utf8');
const val=fs.readFileSync(path.join(root,'docs/05-delivery/validation.md'),'utf8');
const requirementIds=[...req.matchAll(/^\| (R\d{2}) \|/gm)].map(m=>m[1]);
if(requirementIds.length!==19)errors.push(`Expected 19 requirements, got ${requirementIds.length}`);
for(const id of requirementIds)if(!val.includes(id))errors.push(`Missing requirement trace ${id}`);
const adrs=docs.filter(p=>/04-decisions[\\/]\d{4}-.*\.md$/.test(p));
if(adrs.length!==16)errors.push(`Expected 16 ADRs, got ${adrs.length}`);
for(const f of adrs)if(!fs.readFileSync(f,'utf8').includes('Status: Proposed.'))errors.push(`Unexpected ADR status: ${f}`);
const backlog=fs.readFileSync(path.join(root,'docs/05-delivery/backlog.md'),'utf8');
const tasks=[...backlog.matchAll(/^\| (B\d{2}) \| (.*?) \| (.*?) \|/gm)].map(m=>({id:m[1],dependencies:[...m[3].matchAll(/\bB\d{2}\b/g)].map(m=>m[0])}));
const taskMap=new Map(tasks.map(t=>[t.id,t]));
if(tasks.length!==40||taskMap.size!==40)errors.push('Expected 40 distinct backlog tasks');
for(const t of tasks)for(const dep of t.dependencies)if(!taskMap.has(dep))errors.push(`Unknown dependency ${t.id}->${dep}`);
function visit(id,stack=[],seen=new Set()){
 if(stack.includes(id)){errors.push(`Dependency cycle: ${[...stack,id].join(' -> ')}`);return;}
 if(seen.has(id)||!taskMap.has(id))return;seen.add(id);
 for(const dep of taskMap.get(id).dependencies)visit(dep,[...stack,id],seen);
}
for(const t of tasks)visit(t.id);
const releaseDependencies=new Set();
function collect(id){if(releaseDependencies.has(id)||!taskMap.has(id))return;releaseDependencies.add(id);for(const dep of taskMap.get(id).dependencies)collect(dep);}
collect('B35');if(releaseDependencies.has('B39'))errors.push('Optional inference blocks core release');
const report={docFiles:docs.length,markdownFiles:docs.filter(p=>p.endsWith('.md')).length,jsonFiles:json.length,
 words:[...texts.values()].join('\n').split(/\s+/).filter(Boolean).length,sources:sourceIds.length,
 adrs:adrs.length,requirements:requirementIds.length,backlogTasks:tasks.length,
 coreReleaseIndependentOfInference:!releaseDependencies.has('B39'),localLinksChecked:links,errors};
fs.mkdirSync(path.join(root,'work'),{recursive:true});
fs.writeFileSync(path.join(root,'work/doc-audit-results.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
process.exitCode=errors.length?1:0;
