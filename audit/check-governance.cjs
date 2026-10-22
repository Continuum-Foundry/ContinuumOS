// Repository consistency checks only; not legal clearance or a security scanner.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const {localOnlyHistorical} = require('./check-repository-hygiene.cjs');
const root = path.resolve('.');
const errors = [];
const checked = {requiredFiles: 0, evidenceFiles: 0, localOnlyEvidenceFiles: 0, localOnlyEvidenceVerifiedLocally: 0, noticeFiles: 0, registryPackages: 0};
const retention = JSON.parse(fs.readFileSync(path.join(root,'audit/repository-hygiene-baseline.json'),'utf8'));
function assert(condition, message) { if (!condition) errors.push(message); }
function resolveInside(base, name) {
  const resolved = path.resolve(base, name);
  if (!resolved.startsWith(root + path.sep)) throw new Error(`Path leaves repository: ${name}`);
  return resolved;
}
function read(name) { return fs.readFileSync(path.join(root, name), 'utf8'); }
function json(name) { return JSON.parse(read(name)); }
function digest(file) { return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex'); }
function walk(dir) { return fs.readdirSync(dir, {withFileTypes: true}).flatMap(e => e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)]); }
const required = ['LICENSE','NOTICE','LEGAL.md','CONTRIBUTING.md','GOVERNANCE.md','CODE_OF_CONDUCT.md',
  'SECURITY.md','SUPPORT.md','PRIVACY.md','TRADEMARKS.md','SUPPLY_CHAIN.md','THIRD_PARTY_NOTICES.md','RELEASE.md',
  '.editorconfig','.github/CODEOWNERS','.github/PULL_REQUEST_TEMPLATE.md','.github/ISSUE_TEMPLATE/bug_report.yml',
  '.github/ISSUE_TEMPLATE/proposal.yml','.github/ISSUE_TEMPLATE/config.yml',
  'EXPORT_CONTROL.md','.githooks/pre-push','legal/export-classification.json',
  'legal/dependency-inventory.json','legal/notice-provenance.json'];
for (const file of required) { checked.requiredFiles++; assert(fs.existsSync(path.join(root,file)), `Missing policy file: ${file}`); }
if (!errors.length) {
  assert(/all rights reserved/i.test(read('LICENSE')), 'LICENSE must match the reserved-rights decision');
  assert(/not an open-source license/i.test(read('LICENSE')), 'Rights scope missing from LICENSE');
  const owners = read('.github/CODEOWNERS').split(/\r?\n/).filter(l=>l.trim() && !l.startsWith('#'));
  assert(owners.some(l=>/^\*\s+@hgarg1$/.test(l)), 'Verified default owner missing');
  assert(owners.some(l=>/^\/\.github\/\s+@hgarg1$/.test(l)), 'Workflow/owner-file routing missing');
  assert(!fs.existsSync(path.join(root,'.github/dependabot.yml')), 'Remote dependency automation is on hold');
  const workflowDir = path.join(root,'.github/workflows');
  assert(!fs.existsSync(workflowDir) || !walk(workflowDir).some(f=>/\.ya?ml$/.test(f)), 'Remote workflows are on hold');
  const exportRecord = json('legal/export-classification.json');
  assert(exportRecord.remote_transfer_hold === true && exportRecord.approved_remote_environments.length === 0,
    'Remote-transfer hold must remain active; no environment is approved');
  assert(exportRecord.eccn === null && exportRecord.compliance_certified === false, 'Do not invent export classification or certification');
  assert(/exit 1\s*$/.test(read('.githooks/pre-push')), 'Push guard must deny by default');
  assert(/\[net\][\s\S]*?offline\s*=\s*true/.test(read('.cargo/config.toml')), 'Cargo must default to offline mode during the hold');
  assert(!/<img[^>]+src="https?:/i.test(read('README.md')), 'Remote README image requests are on hold');
  for (const match of read('README.md').matchAll(/(?:href|src)="([^"]+)"/g)) {
    const target = match[1];
    if (!/^(https?:|#)/.test(target)) assert(fs.existsSync(resolveInside(root,target.split('#')[0])), `Broken README HTML reference: ${target}`);
  }
  const manifests = ['tools/xtask/Cargo.toml','kernel/Cargo.toml','crates/editor-model/Cargo.toml','crates/text-core/Cargo.toml','crates/custom-ui/Cargo.toml','crates/slint-ui/Cargo.toml','experiments/ui-link/Cargo.toml','experiments/kernel-link/Cargo.toml',
    'experiments/userland-link/Cargo.toml','experiments/dependencies/Cargo.toml','experiments/boot-bindings/Cargo.toml'];
  assert(/license-file = "LICENSE"/.test(read('Cargo.toml')), 'Workspace rights file not declared');
  assert(/publish = false/.test(read('Cargo.toml')), 'Workspace publishing must be disabled');
  for (const manifest of manifests) {
    const text = read(manifest);
    assert(/license-file(?:\.workspace = true| = "\.\.\/\.\.\/LICENSE")/.test(text), `Missing rights declaration: ${manifest}`);
    assert(/publish(?:\.workspace = true| = false)/.test(text), `Publishing policy missing: ${manifest}`);
  }
  const inventory = json('legal/dependency-inventory.json');
  assert(inventory.schema_version === 1 && inventory.distribution_cleared === false, 'Inventory must not claim distribution clearance');
  assert(Array.isArray(inventory.packages) && inventory.packages.length > 0, 'Empty dependency inventory');
  const locks = new Map();
  for (const lock of inventory.lockfiles) {
    const absolute = resolveInside(root,lock.path);
    assert(digest(absolute) === lock.sha256, `Inventory stale: ${lock.path}; regenerate offline inventory`);
    for (const block of fs.readFileSync(absolute,'utf8').split('[[package]]').slice(1)) {
      const field = key => new RegExp(`^${key} = "([^"]+)"`, 'm').exec(block)?.[1];
      if (field('source')) locks.set(`${field('name')}@${field('version')}|${field('source')}`,field('checksum'));
    }
  }
  const inventoryRegistry = new Set();
  for (const pkg of inventory.packages) {
    if (!pkg.first_party) {
      checked.registryPackages++;
      const key = `${pkg.name}@${pkg.version}|${pkg.source}`;
      inventoryRegistry.add(key);
      assert(locks.has(key) && locks.get(key) === pkg.registry_checksum, `Registry identity/checksum mismatch: ${key}`);
      assert(pkg.declared_license || pkg.declared_license_file, `Missing declared terms: ${key}`);
    }
  }
  // Inventory covers the declared target graphs, so platform-only lock entries
  // outside those resolved graphs are allowed and are not called shipped code.
  assert(inventoryRegistry.size === checked.registryPackages, 'Duplicate registry inventory identities');
  for (const input of json('legal/notice-provenance.json').files) {
    const absolute = resolveInside(root,input.path);
    assert(digest(absolute) === input.sha256, `Upstream notice changed: ${input.path}`);
    checked.noticeFiles++;
  }
  for (const index of walk(path.join(root,'evidence')).filter(p=>path.basename(p)==='SHA256SUMS.json')) {
    const manifest = JSON.parse(fs.readFileSync(index,'utf8'));
    assert(manifest.algorithm === 'SHA-256', `Unexpected evidence checksum algorithm: ${index}`);
    for (const entry of manifest.files) {
      const absolute = resolveInside(path.dirname(index),entry.path);
      const relative = path.relative(root,absolute).replaceAll('\\','/');
      if (localOnlyHistorical(relative,retention)) {
        checked.localOnlyEvidenceFiles++;
        // Verify copies when present, but never pretend a fresh checkout has them.
        if (fs.existsSync(absolute)) {
          assert(digest(absolute) === entry.sha256, `Local historical evidence changed: ${relative}`);
          if (entry.bytes !== undefined) assert(fs.statSync(absolute).size === entry.bytes, `Local historical evidence size changed: ${relative}`);
          checked.localOnlyEvidenceVerifiedLocally++;
        }
        continue;
      }
      assert(fs.existsSync(absolute), `Missing frozen evidence: ${entry.path}`);
      if (fs.existsSync(absolute)) {
        assert(digest(absolute) === entry.sha256, `Frozen evidence changed: ${path.relative(root,absolute)}`);
        if (entry.bytes !== undefined) assert(fs.statSync(absolute).size === entry.bytes, `Frozen evidence size changed: ${entry.path}`);
      }
      checked.evidenceFiles++;
    }
  }
}
const report = {status:errors.length ? 'failed':'passed', ...checked, errors,
  limits:'Historical local-only logs/source require preserved local copies or the recorded earlier Git revision. Does not inspect remote GitHub enforcement, authenticate publishers, clear distribution rights, or run kernel/VM tests'};
fs.mkdirSync(path.join(root,'work'),{recursive:true});
fs.writeFileSync(path.join(root,'work/governance-audit-results.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));
process.exitCode = errors.length ? 1 : 0;
