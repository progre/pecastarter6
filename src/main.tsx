import { invoke } from '@tauri-apps/api/core';
import React from 'react';
import { createRoot } from 'react-dom/client';
import Settings from './entities/Settings';
import YPConfig from './entities/YPConfig';
import initFluentUI from './utils/initFluentUI';
import App from './App';

import 'modern-normalize';
import './index.css';

initFluentUI();

async function main() {
  const [ypConfigs, settings, contactStatus] = (await invoke(
    'initial_data'
  )) as any;

  const container = document.getElementById('root');
  const root = createRoot(container!);
  root.render(
    <React.StrictMode>
      <App
        ypConfigs={ypConfigs as readonly YPConfig[]}
        defaultSettings={settings as Settings}
        contactStatus={contactStatus}
      />
    </React.StrictMode>,
  );
}

main();
