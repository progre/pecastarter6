import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api';
import { useState } from 'react';
import { YellowPagesSettings as Settings } from '../entities/Settings';
import TextField from './molecules/TextField';

export default function YellowPagesSettings(props: {
  defaultSettings: Settings;
}) {
  const [settings, setSettings] = useState(props.defaultSettings);

  const onBlur = () => {
    invoke('set_yellow_pages_settings', {
      yellowPagesSettings: settings,
    });
  };

  const update = (newSettings: Partial<Settings>) => {
    setSettings((settings) => ({ ...settings, ...newSettings }));
  };

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 32px;
      `}
      onBlur={onBlur}
    >
      <div
        css={css`
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        `}
      >
        <TextField
          label="IPv4 掲載 YP ホスト"
          type="text"
          value={settings.ipv4YpHost}
          onChangeValue={(value) => update({ ipv4YpHost: value })}
        />
        <TextField
          label="IPv4 掲載 YP 固有設定"
          type="text"
          onChangeValue={(value) => update({ ipv4YpGenrePrefix: value })}
        />
      </div>
      <div
        css={css`
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        `}
      >
        <TextField
          label="IPv6 掲載 YP ホスト"
          type="text"
          onChangeValue={(value) => update({ ipv6YpHost: value })}
        />
        <TextField
          label="IPv6 掲載 YP 固有設定"
          type="text"
          onChangeValue={(value) => update({ ipv6YpGenrePrefix: value })}
        />
      </div>
    </div>
  );
}
