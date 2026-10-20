const fs = require('fs');
const base = 'build/m1/suite-1792532218089714300-19176/';
const a = fs.readFileSync(process.argv[2] || base+'final-1-package/continuum.iso');
const b = fs.readFileSync(process.argv[3] || base+'second-clean-package/continuum.iso');
const sa = JSON.parse(fs.readFileSync(base+'final-1-package/iso-manifest.json')).host_session_id;
const sb = JSON.parse(fs.readFileSync(base+'second-clean-package/iso-manifest.json')).host_session_id;
for(const [v,s] of [[a,sa],[b,sb]]) { let i=0; while((i=v.indexOf(s,i))>=0) { v.fill(48,i,i+64); i+=64; } }
let ranges=[];
for(let i=0;i<Math.max(a.length,b.length);i++) if(a[i]!==b[i]) { const last=ranges.at(-1); if(last&&last.end===i)last.end++;else ranges.push({start:i,end:i+1}); }
console.log(JSON.stringify({lengths:[a.length,b.length],ranges:ranges.map(r=>({...r,sector:Math.floor(r.start/2048),offset:r.start%2048,a:a.subarray(Math.max(0,r.start-16),r.end+16).toString('hex'),b:b.subarray(Math.max(0,r.start-16),r.end+16).toString('hex')}))},null,2));
