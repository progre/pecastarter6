import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api';
import { useState } from 'react';
import {
  EachYellowPagesSettings,
  YellowPagesSettings as Settings,
} from '../entities/Settings';
import YPConfig from '../entities/YPConfig';
import YellowPagesPrefixBuilder from './molecules/YellowPagesPrefixBuilder';

function EachYellowPagesSettingsView({
  protocol,
  ypConfigs,
  usedHostForIPV4,
  value,
  onChange,
}: {
  protocol: 'IPv4' | 'IPv6';
  ypConfigs: readonly YPConfig[];
  usedHostForIPV4?: string;
  value: EachYellowPagesSettings;
  onChange(value: EachYellowPagesSettings): void;
}) {
  const id = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;
  const currentYPConfig = ypConfigs.find((x) => x.host === value.host);
  const conflict =
    currentYPConfig != null && currentYPConfig.host === usedHostForIPV4;
  return (
    <div
      css={css`
        display: flex;
        flex: 1;
        flex-wrap: wrap;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <div
        css={css`
          color: ${!conflict ? 'inherit' : '#ff2800'};
        `}
      >
        <div
          css={css`
            display: flex;
          `}
        >
          <label
            htmlFor={id}
            css={css`
              padding-right: 0.5em;
            `}
          >
            {protocol} 掲載 YP:
          </label>
          <select
            id={id}
            css={css`
              flex-grow: 1;
              color: ${!conflict ? 'inherit' : '#ff2800'};
            `}
            value={ypConfigs.findIndex((x) => x.host === value.host)}
            onChange={(e) =>
              onChange({
                ...value,
                host: ypConfigs[Number(e.target.value)]?.host ?? '',
              })
            }
          >
            <option
              value={-1}
              css={css`
                color: initial;
              `}
            >
              掲載しない
            </option>
            {ypConfigs.map((x, i) => (
              <option
                key={i}
                value={i}
                css={css`
                  color: ${x.host !== usedHostForIPV4 ? 'initial' : '#ff2800'};
                `}
              >
                {x.name}
              </option>
            ))}
          </select>
        </div>
        {!conflict ? null : (
          <span
            css={css`
              background-color: white;
              border: 1px solid gray;
              padding: 4px;
              margin-right: 8px;
              position: absolute;
              font-weight: bold;
            `}
          >
            IPv4 と同じ YP を指定すると
            <span
              css={css`
                white-space: nowrap;
              `}
            >
              チャンネル
            </span>
            を掲載できません
          </span>
        )}
      </div>
      <YellowPagesPrefixBuilder
        config={currentYPConfig ?? null}
        value={value}
        onChange={onChange}
      />
    </div>
  );
}

export default function YellowPagesSettings(props: {
  ypConfigs: readonly YPConfig[];
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
        gap: 16px;
        flex-wrap: wrap;
      `}
      onBlur={onBlur}
    >
      <EachYellowPagesSettingsView
        protocol="IPv4"
        ypConfigs={props.ypConfigs}
        value={settings.ipv4}
        onChange={(ipv4) => update({ ipv4 })}
      />
      <EachYellowPagesSettingsView
        protocol="IPv6"
        ypConfigs={props.ypConfigs}
        usedHostForIPV4={settings.ipv4.host}
        value={settings.ipv6}
        onChange={(ipv6) => update({ ipv6 })}
      />
    </div>
  );
}
