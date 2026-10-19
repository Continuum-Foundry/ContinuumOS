const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const oldDir = 'build/m1/exception-invalid-opcode-1792453036450994000-29220';
const newDir = 'build/m1/exception-invalid-opcode-1792453369322739500-30180';
const load = (dir, name) => JSON.parse(fs.readFileSync(path.join(dir, name), 'utf8'));
const oldInputs = load(oldDir, 'build-inputs.json');
const newInputs = load(newDir, 'build-inputs.json');
const changed = Object.keys(newInputs.sources).filter(key => oldInputs.sources[key] !== newInputs.sources[key]);
if (changed.length !== 1 || changed[0] !== 'kernel/src/exception-entry.S') throw Error('Unexpected source difference');
const before = fs.readFileSync('evidence/phase-1/P1-04/2026-10-19/source/kernel/src/exception-entry.S');
const after = fs.readFileSync('kernel/src/exception-entry.S');
if (before[before.length-1] !== 10 || !before.subarray(0,-1).equals(after)) throw Error('Change is not exactly one removed EOF newline');
const oldManifest = load(oldDir, 'build-manifest.json');
const newManifest = load(newDir, 'build-manifest.json');
const oldElf = fs.readFileSync(path.join(oldDir,'kernel.elf'));
const newElf = fs.readFileSync(path.join(newDir,'kernel.elf'));
const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function normalize(bytes, id) {
  const output = Buffer.from(bytes), needle = Buffer.from(id);
  for (let at=0; (at=output.indexOf(needle,at)) !== -1; at += needle.length) output.fill(0,at,at+needle.length);
  return output;
}
const segments = oldManifest.inspection.segments.map((segment, i) => {
  const current = newManifest.inspection.segments[i];
  if (JSON.stringify(segment) !== JSON.stringify(current)) throw Error('Segment layout changed');
  const oldBytes = oldElf.subarray(segment.offset,segment.offset+segment.file_size);
  const newBytes = newElf.subarray(current.offset,current.offset+current.file_size);
  return {flags:segment.flags,address:segment.address,bytes:segment.file_size,bytes_identical:oldBytes.equals(newBytes),old_sha256:digest(oldBytes),new_sha256:digest(newBytes),equal_after_normalizing_only_embedded_build_id:normalize(oldBytes,oldManifest.build_id).equals(normalize(newBytes,newManifest.build_id))};
});
const oldDisassembly = fs.readFileSync(path.join(oldDir,'llvm-objdump.stdout.log'),'utf8');
const newDisassembly = fs.readFileSync(path.join(newDir,'llvm-objdump.stdout.log'),'utf8');
const wrappers = ['exception_start','exception_load_gdt',...Array.from({length:32},(_,i)=>`exception_${i}`),'exception_unexpected','exception_common','diagnostic_invalid_opcode','diagnostic_page_fault','diagnostic_page_fault_site','diagnostic_panic_report'];
for (const name of wrappers) {
  const expression = new RegExp(`^[0-9a-f]+ <${name}>:\\r?\\n(?:[^\\r\\n]+\\r?\\n)+`,'m');
  const before = oldDisassembly.match(expression)?.[0], after = newDisassembly.match(expression)?.[0];
  if (!before || before !== after) throw Error(`Entry wrapper changed: ${name}`);
}
if (load(newDir,'result.json').status !== 'passed') throw Error('New guest run did not pass');
fs.writeFileSync('work/p1-04-formatting-review.json',JSON.stringify({status:'passed',change:'Exactly one EOF newline removed; assembler tokens unchanged',old_build_id:oldManifest.build_id,new_build_id:newManifest.build_id,changed_inputs:changed,entry_wrappers_identical:wrappers,segments,actual_guest_case:'exception-invalid-opcode',guest:'passed',limits:'Full 19-host/21-VM acceptance remains the preceding frozen source snapshot. This follow-up validates the formatting-only final source, unchanged CPU entry wrappers and one actual fault. Entire ELF/text bytes are not identical: the source-derived build ID changes Rust constant placement/data-address operands.'},null,2)+'\n');
console.log('Formatting comparison and final-source guest check passed');
