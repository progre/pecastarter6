import fs from 'fs';
import { basename } from 'path';
import archiver from 'archiver';

const json = JSON.parse(
  fs.readFileSync(`./src-tauri/tauri.conf.json`, { encoding: 'utf8' })
);
const zipFilename = `${json.productName}_${json.version}_x64_ja-JP.zip`;
const exeFilename = `${json.productName}.exe`;
const pathes = [exeFilename, ...json.bundle.resources].map((x) => `${x}`);

const archive = archiver('zip', { zlib: { level: 9 } });
archive.pipe(fs.createWriteStream(zipFilename));
pathes.forEach((x) => {
  const src = `src-tauri/target/release/${x.replace(/\.\./g, '_up_')}`;
  const name = basename(x);
  if (x.endsWith('/')) {
    archive.directory(src, name);
  } else {
    archive.file(src, { name });
  }
});
['LICENSE', 'README.md'].forEach((name) => {
  archive.file(name, { name });
});
archive.finalize();
