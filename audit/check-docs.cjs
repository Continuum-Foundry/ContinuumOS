const fs = require('node:fs');
const path = require('node:path');
const root = path.resolve('.');
function walk(p) { return fs.readdirSync(p,{withFileTypes:true}).flatMap(d=>d.isDirectory()?walk(path.join(p,d.name)):[path.join(p,d.name)]); }
const docs = walk(path.join(root,'docs'));
const rootMarkdown = fs.readdirSync(root).filter(n=>n.endsWith('.md')).map(n=>path.join(root,n));
const githubMarkdown = fs.existsSync(path.join(root,'.github')) ? walk(path.join(root,'.github')).filter(p=>p.endsWith('.md')) : [];
const md = [...docs.filter(p=>p.endsWith('.md')), ...rootMarkdown, ...githubMarkdown];
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
if(adrs.length!==24)errors.push(`Expected 24 ADRs, got ${adrs.length}`);
for(const f of adrs)if(!fs.readFileSync(f,'utf8').includes('Status: Proposed.'))errors.push(`Unexpected ADR status: ${f}`);
const backlog=fs.readFileSync(path.join(root,'docs/05-delivery/backlog.md'),'utf8');
const tasks=[...backlog.matchAll(/^\| (B\d{2}) \| (.*?) \| (.*?) \|/gm)].map(m=>({id:m[1],dependencies:[...m[3].matchAll(/\bB\d{2}\b/g)].map(m=>m[0])}));
const taskMap=new Map(tasks.map(t=>[t.id,t]));
if(tasks.length!==60||taskMap.size!==60)errors.push('Expected 60 distinct backlog tasks');
for(let n=1;n<=60;n++){const id=`B${String(n).padStart(2,'0')}`;if(!taskMap.has(id))errors.push(`Missing backlog ID ${id}`);}
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
const requiredInstallerTasks=['B41','B42','B43','B44'];
for(const id of requiredInstallerTasks)if(!releaseDependencies.has(id))errors.push(`Missing installer/account release prerequisite ${id}`);
function dependencyClosure(rootId){
 const closure=new Set();
 function add(id){if(closure.has(id)||!taskMap.has(id))return;closure.add(id);for(const dep of taskMap.get(id).dependencies)add(dep);}
 add(rootId);return closure;
}
const desktopDependencies=dependencyClosure('B56');
const pcDependencies=dependencyClosure('B58');
const requiredDesktopTasks=['B35','B31',...Array.from({length:11},(_,i)=>`B${45+i}`)];
const requiredPcTasks=['B56','B57','B42','B43','B44','B50','B52','B54','B55'];
for(const id of requiredDesktopTasks)if(!desktopDependencies.has(id))errors.push(`Missing usable desktop release prerequisite ${id}`);
for(const id of requiredPcTasks)if(!pcDependencies.has(id))errors.push(`Missing named-PC release prerequisite ${id}`);
const optionalResearchTasks=['B27','B29','B30','B39','B59','B60'];
for(const [release,closure] of [['M8',releaseDependencies],['M14',desktopDependencies]]){
 for(const id of optionalResearchTasks)if(closure.has(id))errors.push(`Optional research ${id} blocks ${release}`);
}
for(let n=45;n<=60;n++)if(releaseDependencies.has(`B${n}`))errors.push(`Later roadmap task B${n} unexpectedly blocks M8`);
const milestoneText=fs.readFileSync(path.join(root,'docs/05-delivery/milestones.md'),'utf8');
const milestoneIds=[...milestoneText.matchAll(/^## (M\d+) — /gm)].map(m=>m[1]);
const milestoneSet=new Set(milestoneIds);
if(milestoneIds.length!==22||milestoneSet.size!==22)errors.push('Expected 22 distinct milestones M0–M21');
for(let n=0;n<=21;n++)if(!milestoneSet.has(`M${n}`))errors.push(`Missing milestone M${n}`);
const matrixText=fs.readFileSync(path.join(root,'docs/05-delivery/desktop-feature-matrix.md'),'utf8');
const featureRows=matrixText.split(/\r?\n/).filter(line=>/^\| F\d+ \|/.test(line)).map(line=>line.split('|').slice(1,-1).map(value=>value.trim()));
const featureIds=new Set();
const classifications=new Set(['M8 required','M14 required','named-PC profile','optional research','deferred']);
const matrixTasks=new Set();
for(const row of featureRows){
 if(row.length!==7){errors.push(`Feature matrix row needs seven columns: ${row[0]}`);continue;}
 const [id,feature,owner,milestones,backlogIds,classification,acceptance]=row;
 if(!/^F\d{2}$/.test(id))errors.push(`Invalid feature ID ${id}`);
 if(featureIds.has(id))errors.push(`Duplicate feature ID ${id}`);featureIds.add(id);
 if(!feature||!owner||!acceptance||/\b(?:TBD|TODO)\b/i.test(acceptance))errors.push(`Incomplete feature outcome/owner ${id}`);
 if(!classifications.has(classification))errors.push(`Unknown feature classification ${id}: ${classification}`);
 const refsM=[...milestones.matchAll(/\bM\d+\b/g)].map(m=>m[0]);
 const refsB=[...backlogIds.matchAll(/\bB\d+\b/g)].map(m=>m[0]);
 if(!refsM.length||!refsB.length)errors.push(`Missing feature references ${id}`);
 for(const ref of refsM)if(!milestoneSet.has(ref))errors.push(`Unknown feature milestone ${id}->${ref}`);
 for(const ref of refsB){matrixTasks.add(ref);if(!taskMap.has(ref))errors.push(`Unknown feature task ${id}->${ref}`);}
 const release=classification==='M8 required'?'M8':classification==='M14 required'?'M14':classification==='named-PC profile'?'M15':null;
 const closure=release==='M8'?releaseDependencies:release==='M14'?desktopDependencies:pcDependencies;
 if(release){
  if(!refsM.includes(release))errors.push(`Feature ${id} omits its release milestone ${release}`);
  for(const ref of refsB)if(!closure.has(ref))errors.push(`Feature ${id} task ${ref} is outside ${release} release dependencies`);
 }
 if(classification==='optional research')for(const ref of refsB)if(releaseDependencies.has(ref)||desktopDependencies.has(ref))errors.push(`Optional feature ${id} task ${ref} blocks a VM release`);
}
if(featureRows.length!==36||featureIds.size!==36)errors.push('Expected 36 distinct desktop features');
for(let n=1;n<=36;n++){const id=`F${String(n).padStart(2,'0')}`;if(!featureIds.has(id))errors.push(`Missing desktop feature ${id}`);}
for(const id of ['B27','B29','B30','B31','B39',...Array.from({length:16},(_,i)=>`B${45+i}`)])if(!matrixTasks.has(id))errors.push(`Feature matrix omits roadmap task ${id}`);
const report={docFiles:docs.length,markdownFiles:docs.filter(p=>p.endsWith('.md')).length,jsonFiles:json.length,
 words:[...texts.values()].join('\n').split(/\s+/).filter(Boolean).length,sources:sourceIds.length,
 adrs:adrs.length,requirements:requirementIds.length,backlogTasks:tasks.length,
 coreReleaseIndependentOfInference:!releaseDependencies.has('B39'),
 coreReleaseRequiresInstallerAndAccounts:requiredInstallerTasks.every(id=>releaseDependencies.has(id)),
 desktopReleaseRequiresDeclaredWork:requiredDesktopTasks.every(id=>desktopDependencies.has(id)),
 namedPcRequiresVmReleaseAndQualification:requiredPcTasks.every(id=>pcDependencies.has(id)),
 optionalResearchExcludedFromVmReleases:optionalResearchTasks.every(id=>!releaseDependencies.has(id)&&!desktopDependencies.has(id)),
 milestones:milestoneIds.length,featureRows:featureRows.length,localLinksChecked:links,errors};
fs.mkdirSync(path.join(root,'work'),{recursive:true});
fs.writeFileSync(path.join(root,'work/doc-audit-results.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
process.exitCode=errors.length?1:0;
