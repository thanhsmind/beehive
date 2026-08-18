#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const defaultRoot = path.resolve(scriptDir, '../../..');

function fail(message) {
  // Throw instead of calling process.exit so direct-entry callers and the
  // isolated CLI runner can observe the diagnostic before the process closes.
  throw new Error(message);
}

function slash(filePath) {
  return filePath.split(path.sep).join('/');
}

function relative(root, filePath) {
  return slash(path.relative(root, filePath));
}

function parseArgs(argv) {
  let root = defaultRoot;
  let check = false;
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--check') {
      if (check) fail('duplicate argument: --check');
      check = true;
    } else if (arg === '--root') {
      if (index + 1 >= argv.length || argv[index + 1].startsWith('--')) {
        fail('--root requires a path');
      }
      root = path.resolve(argv[index + 1]);
      index += 1;
    } else {
      fail(`unknown argument: ${arg}`);
    }
  }
  return { root, check };
}

function topLevelKey(line) {
  const match = line.match(/^([A-Za-z0-9_-]+):(?:[ \t]|$)/);
  return match ? match[1] : null;
}

function parseIdentity(root, skillDir) {
  const skillFile = path.join(skillDir, 'SKILL.md');
  const skillLabel = relative(root, skillFile);
  let stat;
  try {
    stat = fs.lstatSync(skillFile);
  } catch {
    throw new Error(`ORPHAN ${relative(root, skillDir)}: missing valid SKILL.md`);
  }
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error(`MALFORMED ${skillLabel}: SKILL.md must be a regular file`);
  }

  const source = fs.readFileSync(skillFile, 'utf8').replace(/\r\n/g, '\n');
  const lines = source.split('\n');
  if (lines[0] !== '---') {
    throw new Error(`MALFORMED ${skillLabel}: missing opening frontmatter delimiter`);
  }
  const close = lines.indexOf('---', 1);
  if (close < 0) {
    throw new Error(`MALFORMED ${skillLabel}: missing closing frontmatter delimiter`);
  }
  const frontmatter = lines.slice(1, close);
  const occurrences = new Map();
  for (let index = 0; index < frontmatter.length; index += 1) {
    const key = topLevelKey(frontmatter[index]);
    if (key) {
      const entries = occurrences.get(key) ?? [];
      entries.push(index);
      occurrences.set(key, entries);
    }
  }

  for (const required of ['name', 'description']) {
    const entries = occurrences.get(required) ?? [];
    if (entries.length === 0) {
      throw new Error(`MALFORMED ${skillLabel}: missing ${required} key`);
    }
    if (entries.length !== 1) {
      throw new Error(`MALFORMED ${skillLabel}: duplicate ${required} key`);
    }
  }

  const nameIndex = occurrences.get('name')[0];
  const nameMatch = frontmatter[nameIndex].match(/^name: ([a-z0-9]+(?:-[a-z0-9]+)*)[ \t]*$/);
  if (!nameMatch) {
    throw new Error(`MALFORMED ${skillLabel}: unsupported name scalar style`);
  }
  const name = nameMatch[1];
  const directoryName = path.basename(skillDir);
  if (name !== directoryName) {
    throw new Error(`MALFORMED ${skillLabel}: name ${name} does not match ${directoryName}`);
  }

  const descriptionIndex = occurrences.get('description')[0];
  if (!/^description: >-[ \t]*$/.test(frontmatter[descriptionIndex])) {
    throw new Error(`MALFORMED ${skillLabel}: unsupported description scalar style (expected >-)`);
  }
  const descriptionLines = [];
  for (let index = descriptionIndex + 1; index < frontmatter.length; index += 1) {
    const line = frontmatter[index];
    if (topLevelKey(line)) break;
    if (line === '') {
      descriptionLines.push('');
      continue;
    }
    const match = line.match(/^ {2,}(.*)$/);
    if (!match) {
      throw new Error(`MALFORMED ${skillLabel}: invalid description indentation`);
    }
    descriptionLines.push(match[1].trim());
  }
  const description = descriptionLines.filter(Boolean).join(' ');
  if (!description) {
    throw new Error(`MALFORMED ${skillLabel}: empty description`);
  }

  return { name, description };
}

function displayName(name) {
  return name
    .split('-')
    .map((token) => token.replace(/^[a-z]/, (letter) => letter.toUpperCase()))
    .join(' ');
}

function render(identity) {
  return [
    'interface:',
    `  display_name: ${JSON.stringify(displayName(identity.name))}`,
    `  short_description: ${JSON.stringify(identity.description)}`,
    'policy:',
    '  allow_implicit_invocation: true',
    '',
  ].join('\n');
}

function metadataFor(root, skillDir) {
  return path.join(skillDir, 'agents', 'openai.yaml');
}

function regularDirectory(root, entry) {
  const location = path.join(root, entry.name);
  const stat = fs.lstatSync(location);
  return stat.isDirectory() && !stat.isSymbolicLink();
}

function main() {
  const { root, check } = parseArgs(process.argv.slice(2));
  const skillsRoot = path.join(root, 'skills');
  let entries;
  try {
    entries = fs.readdirSync(skillsRoot, { withFileTypes: true })
      .filter((entry) => regularDirectory(skillsRoot, entry))
      .sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
  } catch (error) {
    fail(`MALFORMED ${relative(root, skillsRoot)}: ${error instanceof Error ? error.message : String(error)}`);
  }

  const skillDirs = entries.filter((entry) => entry.name.startsWith('bee-'));
  const unexpected = entries
    .filter((entry) => !entry.name.startsWith('bee-'))
    .map((entry) => path.join(skillsRoot, entry.name, 'agents', 'openai.yaml'))
    .filter((metadataFile) => fs.existsSync(metadataFile));
  if (unexpected.length > 0) {
    fail(`ORPHAN ${relative(root, unexpected[0])}: metadata has no bee-* source skill`);
  }

  const projections = [];
  try {
    for (const entry of skillDirs) {
      const skillDir = path.join(skillsRoot, entry.name);
      const identity = parseIdentity(root, skillDir);
      projections.push({
        metadataFile: metadataFor(root, skillDir),
        content: render(identity),
      });
    }
  } catch (error) {
    fail(error instanceof Error ? error.message : String(error));
  }

  let failures = 0;
  for (const projection of projections) {
    const label = relative(root, projection.metadataFile);
    if (check) {
      if (!fs.existsSync(projection.metadataFile)) {
        throw new Error(`MISSING ${label}`);
      }
      const stat = fs.lstatSync(projection.metadataFile);
      if (!stat.isFile() || stat.isSymbolicLink()) {
        throw new Error(`STALE ${label}`);
      }
      if (fs.readFileSync(projection.metadataFile, 'utf8') !== projection.content) {
        throw new Error(`STALE ${label}`);
      }
    } else {
      fs.mkdirSync(path.dirname(projection.metadataFile), { recursive: true });
      fs.writeFileSync(projection.metadataFile, projection.content, 'utf8');
    }
  }

  if (failures > 0) {
    // Let stderr flush so callers running this as a child process receive the
    // concrete stale/missing path instead of an empty diagnostic.
    process.exitCode = 1;
    return;
  }
  process.stdout.write(`${check ? 'Checked' : 'Rendered'} ${projections.length} OpenAI metadata projections\n`);
}

main();
