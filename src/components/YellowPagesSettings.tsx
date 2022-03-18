import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api';
import { open } from '@tauri-apps/api/shell';
import { useState } from 'react';
import {
  EachYellowPagesSettings,
  YellowPagesSettings as Settings,
} from '../entities/Settings';
import YPConfig from '../entities/YPConfig';
import TermsCheckbox from './molecules/TermsCheckbox';
import YellowPagesPrefixBuilder from './molecules/YellowPagesPrefixBuilder';
import YPConflictWarning from './molecules/YPConflictWarning';
import YPSelect from './molecules/YPSelect';

function EachYellowPagesSettingsView({
  protocol,
  ypConfigs,
  usedHostForIPV4,
  agreedTerms,
  value,
  onChange,
  onChangeAgreeTerms,
}: {
  protocol: 'IPv4' | 'IPv6';
  ypConfigs: readonly YPConfig[];
  usedHostForIPV4?: string;
  agreedTerms: { [url: string]: string };
  value: EachYellowPagesSettings;
  onChange(value: EachYellowPagesSettings): void;
  onChangeAgreeTerms(url: string, hash: string | null): void;
}): JSX.Element {
  const id = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;
  const currentYPConfig = ypConfigs.find((x) => x.host === value.host);
  const conflict =
    currentYPConfig != null && currentYPConfig.host === usedHostForIPV4;
  const [readedTerms, setReadedTerms] = useState<string | null>();
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
          <YPSelect
            id={id}
            ypConfigs={ypConfigs}
            usedHostForIPV4={usedHostForIPV4}
            conflict={conflict}
            host={value.host}
            onChange={(host) => {
              setReadedTerms(null);
              onChange({ ...value, host });
            }}
          />
        </div>
        {!conflict ? null : <YPConflictWarning />}
      </div>
      <TermsCheckbox
        termsURL={currentYPConfig?.termsURL ?? null}
        readed={readedTerms != null}
        agreed={agreedTerms[currentYPConfig?.termsURL ?? ''] != null}
        onClickReadTerms={async () => {
          const termsURL = currentYPConfig?.termsURL ?? '';
          open(termsURL);
          const termsHash: string = await invoke('fetch_hash', {
            url: termsURL,
          });
          setReadedTerms(termsHash);
        }}
        onChangeAgreeTerms={(value) =>
          onChangeAgreeTerms(
            currentYPConfig?.termsURL ?? '',
            value ? readedTerms!! : null
          )
        }
      />
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
        agreedTerms={settings.agreedTerms}
        value={settings.ipv4}
        onChange={(ipv4) => update({ ipv4 })}
        onChangeAgreeTerms={(url, hash) =>
          update({
            agreedTerms: {
              ...settings.agreedTerms,
              [url]: hash ?? undefined!!,
            },
          })
        }
      />
      <EachYellowPagesSettingsView
        protocol="IPv6"
        ypConfigs={props.ypConfigs}
        usedHostForIPV4={settings.ipv4.host}
        agreedTerms={settings.agreedTerms}
        value={settings.ipv6}
        onChange={(ipv6) => update({ ipv6 })}
        onChangeAgreeTerms={(url, hash) =>
          update({
            agreedTerms: {
              ...settings.agreedTerms,
              [url]: hash ?? undefined!!,
            },
          })
        }
      />
    </div>
  );
}
