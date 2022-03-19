#!/bin/bash -eux

import fs from 'fs';
import childProcess from 'child_process';

const json = JSON.parse(
  fs.readFileSync(`./src-tauri/tauri.conf.json`, { encoding: 'utf8' })
);
const zipFilename = `${json.package.productName}_${json.package.version}.zip`;
const exePath = `./src-tauri/target/release/${json.package.productName}.exe`;
const resources = json.tauri.bundle.resources
  .map((x) => `"./src-tauri/target/release/${x}"`)
  .join(' ');

childProcess.execSync(
  `zip --recurse-paths "${zipFilename}" "${exePath}" ${resources}`,
  { encoding: 'utf8' }
);
