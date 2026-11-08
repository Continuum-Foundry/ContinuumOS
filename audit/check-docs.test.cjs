'use strict';
// Run the real auditor with read-only in-memory mutations; never edit repository
// inputs or overwrite its positive report with expected negative results.
const test=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const source=fs.readFileSync(path.join(__dirname,'check-docs.cjs'),'utf8');
function audit(file,mutate){
 const target=file&&path.resolve(file); let report;
 const mockFs={...fs,readFileSync:(p,...args)=>{
  const raw=fs.readFileSync(p,...args);return target&&path.resolve(p)===target?mutate(raw):raw;
 },mkdirSync:()=>{},writeFileSync:()=>{}};
 const proc={cwd:()=>process.cwd(),exitCode:0};
 vm.runInNewContext(source,{require:name=>name==='node:fs'?mockFs:require(name),process:proc,console:{log:raw=>{report=JSON.parse(raw);}}});
 return {exitCode:proc.exitCode,report};
}
function changeRow(text,id,from,to){
 const row=text.split(/\r?\n/).find(line=>line.startsWith(`| ${id} |`));
 assert.ok(row&&row.includes(from),`Mutation precondition ${id}`);
 return text.replace(row,row.replace(from,to));
}
function rejects(name,file,mutate,expected){test(name,()=>{
 const result=audit(file,mutate);assert.equal(result.exitCode,1);
 assert.ok(result.report.errors.includes(expected),JSON.stringify(result.report.errors));
});}
test('current plan has complete traces and unchanged release separation',()=>{
 const result=audit();assert.equal(result.exitCode,0);assert.deepEqual(result.report.errors,[]);
 assert.equal(result.report.aiConcerns,21);assert.equal(result.report.backlogTasks,89);
 assert.equal(result.report.conditionalAiExcludedFromExistingReleases,true);
 assert.equal(result.report.authorizationExcludedFromExistingReleases,true);
});
const backlog='docs/05-delivery/backlog.md';
for(const [root,release] of [['B35','M8'],['B56','M14'],['B66','M22'],['B58','M15']]){
 rejects(`Authorization cannot become a hidden ${release} prerequisite`,backlog,
  text=>changeRow(text,root,' | B',' | B83,B'),`Authorization profile task B83 blocks ${release}`);
}
rejects('M7 accounts cannot require Cedar evaluation',backlog,
 text=>changeRow(text,'B41',' | B02,',' | B83,B02,'),'Authorization amendment changes existing gate B41->B83');
rejects('enterprise authorization cannot require AI federation',backlog,
 text=>changeRow(text,'B85',' | B41,',' | B78,B41,'),'Authorization gate B85 requires AI task B78');
rejects('data-center authorization requires workload enforcement',backlog,
 text=>changeRow(text,'B89','B87,B88','B87'),'Missing authorization gate prerequisite B89->B88');
for(const [root,release] of [['B35','M8'],['B56','M14'],['B66','M22'],['B58','M15']]){
 rejects(`AI cannot become a hidden ${release} prerequisite`,backlog,
  text=>changeRow(text,root,' | B',' | B67,B'),`Conditional AI task B67 blocks ${release}`);
}
rejects('local demo cannot omit bounded budget admission',backlog,
 text=>changeRow(text,'B39','B67,B68','B67'),'Missing AI gate prerequisite B39->B68');
rejects('private dispatch needs durable accounting',backlog,
 text=>changeRow(text,'B72','B70,B81,B82','B70,B82'),'Missing AI gate prerequisite B72->B81');
rejects('public dispatch needs credential brokerage',backlog,
 text=>changeRow(text,'B73','B81,B82,B53','B81,B53'),'Missing AI gate prerequisite B73->B82');
rejects('remote paths cannot depend on a successful Gemma port',backlog,
 text=>changeRow(text,'B69',' | B21,',' | B39,B21,'),'Route-independent AI gate B72 requires native Gemma qualification');
rejects('public routes cannot force private-model implementation',backlog,
 text=>changeRow(text,'B73',' | B69,',' | B72,B69,'),'Public inference unnecessarily requires private inference implementation');
rejects('strict-offline personal profiles cannot force remote credentials',backlog,
 text=>changeRow(text,'B74','B81,B32','B81,B82,B32'),'Common personal profile forces optional route prerequisite B82');
rejects('cooperation cannot force a private route',backlog,
 text=>changeRow(text,'B75',' | B67,',' | B72,B67,'),'Common cooperation gate B75 forces route B72');
const qualification='docs/06-contracts/examples/ai-qualification-gates.json';
rejects('private routes cannot lose their additional qualification gate',qualification,text=>{
 const fixture=JSON.parse(text);fixture.route_gates.private=[];return JSON.stringify(fixture);
},'Missing conditional AI qualification private->B72');
rejects('local-model remote tools still need a durable ledger',qualification,text=>{
 const fixture=JSON.parse(text);fixture.external_tool_prerequisites=fixture.external_tool_prerequisites.filter(id=>id!=='B81');return JSON.stringify(fixture);
},'External AI tools lack network, durable-accounting or credential prerequisites');
rejects('a narrower profile cannot redefine complete program coverage',qualification,text=>{
 const fixture=JSON.parse(text);fixture.full_program_routes=['public'];return JSON.stringify(fixture);
},'Full AI program omits a planned route class');
rejects('every researched concern remains traced','docs/05-delivery/ai-roadmap.md',
 text=>text.replace(/^\| 21 .*\r?\n/m,''),'Missing AI concern 21');
rejects('local profile capacity cannot raise cold-idle baseline','docs/00-overview/requirements.md',
 text=>text.replace('desktop budget: 256 MiB','desktop budget: 512 MiB'),'The 256 MiB model-unloaded desktop baseline changed or disappeared');
rejects('larger local profiles cannot silently relax the old object parser bound','docs/06-contracts/schemas/package-manifest.schema.json',text=>{
 const schema=JSON.parse(text);schema.properties.files.items.properties.bytes.maximum=4294967296;return JSON.stringify(schema);
},'Legacy v1 package object bound changed or disappeared');
rejects('larger local profiles cannot silently relax old package memory admission','docs/06-contracts/schemas/package-manifest.schema.json',text=>{
 const schema=JSON.parse(text);schema.properties.limits.properties.memory_bytes.maximum=4294967296;return JSON.stringify(schema);
},'Legacy v1 package memory bound changed or disappeared');
rejects('native loading needs an explicit large-artifact contract gate',qualification,text=>{
 const fixture=JSON.parse(text);delete fixture.large_model_contract_gate;return JSON.stringify(fixture);
},'Missing bounded large-model contract gate or unchanged legacy limits');
