import { invoke } from '@tauri-apps/api';
import React from 'react';
import ReactDOM from 'react-dom';
import Settings from './entities/Settings';
import YPConfig from './entities/YPConfig';
import initFluentUI from './utils/initFluentUI';
import App from './App';

import './index.css';

initFluentUI();

async function main() {
  const [ypConfigs, settings] = await invoke('initial_data');

  ReactDOM.render(
    <React.StrictMode>
      <App
        ypConfigs={ypConfigs as readonly YPConfig[]}
        defaultSettings={settings as Settings}
      />
    </React.StrictMode>,
    document.getElementById('root')
  );
}

main();
