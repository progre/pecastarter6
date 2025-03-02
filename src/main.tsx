import { invoke } from '@tauri-apps/api/core';
import React from 'react';
import ReactDOM from 'react-dom';
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

  ReactDOM.render(
    <React.StrictMode>
      <App
        ypConfigs={ypConfigs as readonly YPConfig[]}
        defaultSettings={settings as Settings}
        contactStatus={contactStatus}
      />
    </React.StrictMode>,
    document.getElementById('root')
  );
}

main();
