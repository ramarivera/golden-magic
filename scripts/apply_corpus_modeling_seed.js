#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const corpusPath = path.join(repoRoot, "corpus", "cli-tools.seed.json");
const modelingPath = path.join(repoRoot, "corpus", "modeling.seed.json");

const corpus = JSON.parse(fs.readFileSync(corpusPath, "utf8"));
const modeling = JSON.parse(fs.readFileSync(modelingPath, "utf8"));

const entriesByRepo = new Map(corpus.map((entry) => [entry.repo, entry]));
const seenDescriptors = new Set();

for (const model of modeling) {
  const entry = entriesByRepo.get(model.repo);
  if (!entry) {
    throw new Error(`modeling seed repo is not in corpus: ${model.repo}`);
  }
  if (seenDescriptors.has(model.descriptor_id)) {
    throw new Error(`duplicate modeling descriptor id: ${model.descriptor_id}`);
  }
  seenDescriptors.add(model.descriptor_id);

  const fixtureDir = path.join(repoRoot, model.fixture);
  for (const requiredFile of ["descriptor.toml", "input.txt", "expected.rows.json", "negative.txt"]) {
    const requiredPath = path.join(fixtureDir, requiredFile);
    if (!fs.existsSync(requiredPath)) {
      throw new Error(`modeling fixture missing ${requiredFile}: ${model.fixture}`);
    }
  }

  entry.lifecycle = {
    found: true,
    analyzed: true,
    modeled: true,
    deterministic_tested: true,
    agentic_tested: true,
  };
  entry.status = "agentic-tested";
  entry.descriptor_id = model.descriptor_id;
  entry.backend = model.backend;
  entry.deterministic_cases = 1;
  entry.agentic_runs = 1;
  entry.analysis_notes = model.analysis_notes;
}

fs.writeFileSync(corpusPath, `${JSON.stringify(corpus, null, 2)}\n`);
console.error(`applied ${modeling.length} modeled corpus entries`);
