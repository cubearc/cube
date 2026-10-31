import { readFile, rename, rm, writeFile } from 'node:fs/promises';
import { createCube } from '../dist/node.mjs';
import * as wasm from '../dist/wasm/cube_wasm.js';
import { qualificationReport, sdkDriver } from './qualification-library.mjs';

const destination = process.argv[2];
if (!destination) throw new Error('Provide an explicit output path for the qualification report.');
// A failed rerun must never leave an old passed report at the requested path.
await rm(destination, { force: true });
const temporary = `${destination}.${process.pid}.tmp`;
try {
  const cube = await createCube();
  const report = await qualificationReport({
    snapshot: JSON.parse(wasm.catalog_snapshot_json()),
    canonicalJson: await readFile(new URL('../../../contracts/v1/catalog.json', import.meta.url), 'utf8'),
    fixtures: JSON.parse(await readFile(new URL('../../../crates/cube-core/src/operations/contracts/conformance.json', import.meta.url), 'utf8')),
    build: cube.build, executors: cube.executors, adapters: cube.pipelineAdapters, limits: cube.limits,
    runtime: { kind: 'node-wasm', version: process.version, platform: process.platform, architecture: process.arch },
    driver: sdkDriver(cube),
  });
  await writeFile(temporary, `${JSON.stringify(report, null, 2)}\n`);
  await rename(temporary, destination);
  console.log(`${report.catalog.operationCount} contracts verified; ${report.coverage.direct} direct and ${report.coverage.pipeline} pipeline paths executed in ${process.version}.`);
} finally { await rm(temporary, { force: true }); }
