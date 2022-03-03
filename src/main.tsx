import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';
import { invoke } from '@tauri-apps/api';
import Settings from './entities/Settings';

invoke('initial_settings').then((settings) => {
  ReactDOM.render(
    <React.StrictMode>
      <App settings={settings as Settings} />
    </React.StrictMode>,
    document.getElementById('root')
  );
});
