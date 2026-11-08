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
if(requirementIds.length!==27||new Set(requirementIds).size!==27)errors.push(`Expected 27 distinct requirements, got ${requirementIds.length}`);
for(let n=1;n<=27;n++)if(!requirementIds.includes(`R${String(n).padStart(2,'0')}`))errors.push(`Missing requirement R${String(n).padStart(2,'0')}`);
for(const id of requirementIds)if(!val.includes(id))errors.push(`Missing requirement trace ${id}`);
if(!/\*\*cold-idle\*\* desktop budget:\s*256 MiB resident\/committed guest memory/.test(req))errors.push('The 256 MiB model-unloaded desktop baseline changed or disappeared');
const adrs=docs.filter(p=>/04-decisions[\\/]\d{4}-.*\.md$/.test(p));
if(adrs.length!==34)errors.push(`Expected 34 ADRs, got ${adrs.length}`);
for(const f of adrs)if(!fs.readFileSync(f,'utf8').includes('Status: Proposed.'))errors.push(`Unexpected ADR status: ${f}`);
const backlog=fs.readFileSync(path.join(root,'docs/05-delivery/backlog.md'),'utf8');
const tasks=[...backlog.matchAll(/^\| (B\d{2}) \| (.*?) \| (.*?) \|/gm)].map(m=>({id:m[1],dependencies:[...m[3].matchAll(/\bB\d{2}\b/g)].map(m=>m[0])}));
const taskMap=new Map(tasks.map(t=>[t.id,t]));
if(tasks.length!==89||taskMap.size!==89)errors.push('Expected 89 distinct backlog tasks');
for(let n=1;n<=89;n++){const id=`B${String(n).padStart(2,'0')}`;if(!taskMap.has(id))errors.push(`Missing backlog ID ${id}`);}
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
const displayDependencies=dependencyClosure('B66');
const aiTasks=['B39',...Array.from({length:16},(_,i)=>`B${67+i}`)];
const authorizationTasks=Array.from({length:7},(_,i)=>`B${83+i}`);
for(const [release,closure] of [['M8',releaseDependencies],['M14',desktopDependencies],['M22',displayDependencies],['M15',pcDependencies]]){
 for(const id of authorizationTasks)if(closure.has(id))errors.push(`Authorization profile task ${id} blocks ${release}`);
}
// New authorization work must not become an implicit prerequisite of any
// existing B01-B82 gate, including M7 accounts and the current Phase 4 work.
for(let n=1;n<=82;n++){
 const gate=`B${String(n).padStart(2,'0')}`;
 for(const id of authorizationTasks)if(dependencyClosure(gate).has(id))errors.push(`Authorization amendment changes existing gate ${gate}->${id}`);
}
const authorizationPrerequisites={
 B83:['B13','B37'],B84:['B41','B51','B83'],B85:['B41','B53','B83'],
 B86:['B49','B51','B84','B85'],B87:['B32','B84','B85','B86'],
 B88:['B49','B53','B83','B85','B86'],B89:['B32','B87','B88']
};
for(const [gate,prerequisites] of Object.entries(authorizationPrerequisites)){
 const closure=dependencyClosure(gate);
 for(const id of prerequisites)if(!closure.has(id))errors.push(`Missing authorization gate prerequisite ${gate}->${id}`);
 for(const id of aiTasks)if(closure.has(id))errors.push(`Authorization gate ${gate} requires AI task ${id}`);
}
for(const [release,closure] of [['M8',releaseDependencies],['M14',desktopDependencies],['M22',displayDependencies],['M15',pcDependencies]]){
 for(const id of aiTasks)if(closure.has(id))errors.push(`Conditional AI task ${id} blocks ${release}`);
}
const aiGatePrerequisites={
 B39:['B06','B21','B37','B67','B68'],
 B69:['B21','B23','B25','B41','B52','B67','B68'],
 B70:['B49','B67'],
 B72:['B21','B69','B70','B81','B82','B53','B54'],
 B73:['B21','B69','B70','B81','B82','B53','B54'],
 B74:['B69','B70','B71','B81'], B75:['B69','B81'],
 B76:['B74','B75'], B78:['B72','B75','B77','B81','B82'],
 B79:['B76','B77','B78'], B80:['B67','B77'],
 B81:['B23','B25','B41','B52','B68'], B82:['B41','B49','B53','B67']
};
for(const [gate,prerequisites] of Object.entries(aiGatePrerequisites)){
 const closure=dependencyClosure(gate);
 for(const id of prerequisites)if(!closure.has(id))errors.push(`Missing AI gate prerequisite ${gate}->${id}`);
}
const localAiDependencies=dependencyClosure('B39');
for(const id of ['B69','B70','B71','B72','B73','B74','B75','B76','B78','B79','B80','B81','B82'])if(localAiDependencies.has(id))errors.push(`Early local gate unexpectedly requires later AI task ${id}`);
for(const gate of ['B69','B70','B71','B72','B73','B74','B75','B76','B81','B82'])if(dependencyClosure(gate).has('B39'))errors.push(`Route-independent AI gate ${gate} requires native Gemma qualification`);
if(dependencyClosure('B73').has('B72'))errors.push('Public inference unnecessarily requires private inference implementation');
for(const id of ['B39','B53','B54','B72','B73','B82'])if(dependencyClosure('B74').has(id))errors.push(`Common personal profile forces optional route prerequisite ${id}`);
for(const gate of ['B75','B76'])for(const id of ['B39','B72','B73'])if(dependencyClosure(gate).has(id))errors.push(`Common cooperation gate ${gate} forces route ${id}`);
const aiQualification=JSON.parse(fs.readFileSync(path.join(root,'docs/06-contracts/examples/ai-qualification-gates.json'),'utf8'));
if(aiQualification.schema_version!==1||!aiQualification.status?.startsWith('planning-only')||aiQualification.base_gate!=='B74')errors.push('AI qualification fixture is not the declared planning-only base gate');
const routeGates={local:'B39',private:'B72',public:'B73'};
if(Object.keys(aiQualification.route_gates||{}).sort().join(',')!==Object.keys(routeGates).sort().join(','))errors.push('AI qualification route classes changed or disappeared');
for(const [route,gate] of Object.entries(routeGates)){
 const actual=aiQualification.route_gates?.[route];
 if(!Array.isArray(actual)||actual.length!==1||actual[0]!==gate)errors.push(`Missing conditional AI qualification ${route}->${gate}`);
}
const externalPrerequisites=['B53','B54','B81','B82'];
const legacyPackage=JSON.parse(fs.readFileSync(path.join(root,'docs/06-contracts/schemas/package-manifest.schema.json'),'utf8'));
if(legacyPackage.properties?.files?.items?.properties?.bytes?.maximum!==67108864)errors.push('Legacy v1 package object bound changed or disappeared');
if(legacyPackage.properties?.limits?.properties?.memory_bytes?.maximum!==2147483648)errors.push('Legacy v1 package memory bound changed or disappeared');
if(aiQualification.large_model_contract_gate!=='B67'||aiQualification.existing_object_max_bytes!==67108864||aiQualification.existing_package_memory_max_bytes!==2147483648)errors.push('Missing bounded large-model contract gate or unchanged legacy limits');
if(!Array.isArray(aiQualification.external_tool_prerequisites)||aiQualification.external_tool_prerequisites.length!==externalPrerequisites.length||!externalPrerequisites.every(id=>aiQualification.external_tool_prerequisites.includes(id)))errors.push('External AI tools lack network, durable-accounting or credential prerequisites');
if(!Array.isArray(aiQualification.full_program_routes)||aiQualification.full_program_routes.length!==3||!Object.keys(routeGates).every(route=>aiQualification.full_program_routes.includes(route)))errors.push('Full AI program omits a planned route class');
for(const id of ['B56','B61','B62','B63','B64','B65'])if(!displayDependencies.has(id))errors.push(`Missing display gate prerequisite ${id}`);
for(let n=61;n<=66;n++)if(releaseDependencies.has(`B${n}`)||desktopDependencies.has(`B${n}`))errors.push(`Display expansion B${n} unexpectedly blocks M8/M14`);
for(const id of ['B27','B29','B30','B39','B59','B60'])if(displayDependencies.has(id))errors.push(`Optional research ${id} blocks M22`);
const requiredDesktopTasks=['B35','B31',...Array.from({length:11},(_,i)=>`B${45+i}`)];
const requiredPcTasks=['B66','B56','B57','B42','B43','B44','B50','B52','B54','B55'];
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
if(milestoneIds.length!==29||milestoneSet.size!==29)errors.push('Expected 29 distinct milestones M0 through M28');
for(let n=0;n<=28;n++)if(!milestoneSet.has(`M${n}`))errors.push(`Missing milestone M${n}`);
const matrixText=fs.readFileSync(path.join(root,'docs/05-delivery/desktop-feature-matrix.md'),'utf8');
const featureRows=matrixText.split(/\r?\n/).filter(line=>/^\| F\d+ \|/.test(line)).map(line=>line.split('|').slice(1,-1).map(value=>value.trim()));
const featureIds=new Set();
const classifications=new Set(['M8 required','M14 required','named-PC profile','M22 required','AI profile','authorization profile','optional research','deferred']);
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
 const release=classification==='M8 required'?'M8':classification==='M14 required'?'M14':classification==='named-PC profile'?'M15':classification==='M22 required'?'M22':null;
 const closure=release==='M8'?releaseDependencies:release==='M14'?desktopDependencies:release==='M22'?displayDependencies:pcDependencies;
 if(release){
  if(!refsM.includes(release))errors.push(`Feature ${id} omits its release milestone ${release}`);
  for(const ref of refsB)if(!closure.has(ref))errors.push(`Feature ${id} task ${ref} is outside ${release} release dependencies`);
 }
 if(classification==='optional research')for(const ref of refsB)if(releaseDependencies.has(ref)||desktopDependencies.has(ref))errors.push(`Optional feature ${id} task ${ref} blocks a VM release`);
 if(classification==='AI profile')for(const ref of refsB)if(releaseDependencies.has(ref)||desktopDependencies.has(ref)||displayDependencies.has(ref)||pcDependencies.has(ref))errors.push(`AI feature ${id} task ${ref} blocks an existing release`);
 if(classification==='authorization profile')for(const ref of refsB)if(releaseDependencies.has(ref)||desktopDependencies.has(ref)||displayDependencies.has(ref)||pcDependencies.has(ref))errors.push(`Authorization feature ${id} task ${ref} blocks an existing release`);
}
if(featureRows.length!==47||featureIds.size!==47)errors.push('Expected 47 distinct desktop features');
for(let n=1;n<=47;n++){const id=`F${String(n).padStart(2,'0')}`;if(!featureIds.has(id))errors.push(`Missing desktop feature ${id}`);}
for(const id of ['B27','B29','B30','B31','B39',...Array.from({length:45},(_,i)=>`B${45+i}`)])if(!matrixTasks.has(id))errors.push(`Feature matrix omits roadmap task ${id}`);
const aiRoadmap=fs.readFileSync(path.join(root,'docs/05-delivery/ai-roadmap.md'),'utf8');
const concernRows=aiRoadmap.split(/\r?\n/).filter(line=>/^\| \d+ /.test(line)).map(line=>line.split('|').slice(1,-1).map(value=>value.trim()));
const concerns=new Set();
for(const row of concernRows){
 if(row.length!==4){errors.push(`AI concern row needs four columns: ${row[0]}`);continue;}
 const id=Number.parseInt(row[0],10);if(concerns.has(id))errors.push(`Duplicate AI concern ${id}`);concerns.add(id);
 if(!/\bR\d{2}\b/.test(row[1])||!/\bB\d{2}\b/.test(row[2])||!row[3])errors.push(`Incomplete AI concern trace ${id}`);
 for(const match of row[1].matchAll(/\bR\d{2}\b/g))if(!requirementIds.includes(match[0]))errors.push(`Unknown AI requirement ${match[0]}`);
 for(const match of row[2].matchAll(/\bB\d{2}\b/g))if(!taskMap.has(match[0]))errors.push(`Unknown AI task ${match[0]}`);
}
if(concernRows.length!==21||concerns.size!==21)errors.push('Expected 21 distinct traced AI concerns');
for(let n=1;n<=21;n++)if(!concerns.has(n))errors.push(`Missing AI concern ${n}`);
const report={docFiles:docs.length,markdownFiles:docs.filter(p=>p.endsWith('.md')).length,jsonFiles:json.length,
 words:[...texts.values()].join('\n').split(/\s+/).filter(Boolean).length,sources:sourceIds.length,
 adrs:adrs.length,requirements:requirementIds.length,backlogTasks:tasks.length,
 coreReleaseIndependentOfInference:!releaseDependencies.has('B39'),
 coreReleaseRequiresInstallerAndAccounts:requiredInstallerTasks.every(id=>releaseDependencies.has(id)),
 desktopReleaseRequiresDeclaredWork:requiredDesktopTasks.every(id=>desktopDependencies.has(id)),
 namedPcRequiresVmReleaseAndQualification:requiredPcTasks.every(id=>pcDependencies.has(id)),
 optionalResearchExcludedFromVmReleases:optionalResearchTasks.every(id=>!releaseDependencies.has(id)&&!desktopDependencies.has(id)),
 conditionalAiExcludedFromExistingReleases:aiTasks.every(id=>![releaseDependencies,desktopDependencies,displayDependencies,pcDependencies].some(closure=>closure.has(id))),
 authorizationExcludedFromExistingReleases:authorizationTasks.every(id=>![releaseDependencies,desktopDependencies,displayDependencies,pcDependencies].some(closure=>closure.has(id))),
 aiConcerns:concernRows.length,
 milestones:milestoneIds.length,featureRows:featureRows.length,localLinksChecked:links,errors};
fs.mkdirSync(path.join(root,'work'),{recursive:true});
fs.writeFileSync(path.join(root,'work/doc-audit-results.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
process.exitCode=errors.length?1:0;
