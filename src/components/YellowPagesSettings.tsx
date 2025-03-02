import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';
import {
  EachYellowPagesSettings,
  YellowPagesSettings as Settings,
} from '../entities/Settings';
import YPConfig from '../entities/YPConfig';
import TermsCheckbox from './molecules/TermsCheckbox';
import YellowPagesPrefixBuilder from './molecules/YellowPagesPrefixBuilder';
import YPConflictWarning from './molecules/YPConflictWarning';
import YPSelect from './molecules/YPSelect';

function EachYellowPagesSettingsView(props: {
  protocol: 'IPv4' | 'IPv6';
  ypConfigs: readonly YPConfig[];
  usedHostForIPV4?: string;
  agreedTerms: { [url: string]: string };
  readedTerms: { [url: string]: string };
  value: EachYellowPagesSettings;
  onReadTerms: (termsURL: string, hash: string) => void;
  onChange(value: EachYellowPagesSettings): void;
  onChangeAgreeTerms(url: string, hash: string | null): void;
}): JSX.Element {
  const currentYPConfig = props.ypConfigs.find(
    (x) => x.host === props.value.host
  );
  const conflict =
    currentYPConfig != null && currentYPConfig.host === props.usedHostForIPV4;
  const readedTerms: string | null =
    props.readedTerms[currentYPConfig?.termsURL ?? ''] ?? null;
  return (
    <div
      css={css`
        display: flex;
        flex: 1;
        flex-wrap: wrap;
        flex-direction: column;
        gap: 8px;
        min-width: 210px;
      `}
    >
      <div
        css={css`
          color: ${!conflict ? 'inherit' : '#ff2800'};
        `}
      >
        <YPSelect
          label={`${props.protocol} 掲載 YP`}
          ypConfigs={props.ypConfigs}
          usedHostForIPV4={props.usedHostForIPV4}
          conflict={conflict}
          host={props.value.host}
          onChange={(host) => {
            props.onChange({ ...props.value, host });
          }}
        />
        {!conflict ? null : <YPConflictWarning />}
      </div>
      <TermsCheckbox
        termsURL={currentYPConfig?.termsURL ?? null}
        readed={readedTerms != null}
        agreed={props.agreedTerms[currentYPConfig?.termsURL ?? ''] != null}
        onClickReadTerms={async () => {
          const termsURL = currentYPConfig?.termsURL ?? '';
          open(termsURL);
          const termsHash: string = await invoke('fetch_hash', {
            url: termsURL,
            selector: currentYPConfig?.termsSelector,
          });
          props.onReadTerms(termsURL, termsHash);
        }}
        onChangeAgreeTerms={(value) =>
          props.onChangeAgreeTerms(
            currentYPConfig?.termsURL ?? '',
            value ? readedTerms!! : null
          )
        }
      />
      {!(currentYPConfig?.ignoreTermsCheck === true) ? null : (
        <div
          css={css`
            font-size: x-small;
            padding: 4px;
            margin-top: -8px;
            background-color: #ffff99;
          `}
        >
          この YP
          は利用規約の自動確認に対応していません。規約の更新は自身で確認してください。
        </div>
      )}
      <div
        css={css`
          margin-top: 20px;
        `}
      >
        <YellowPagesPrefixBuilder
          config={currentYPConfig ?? null}
          value={props.value}
          onChange={props.onChange}
        />
      </div>
    </div>
  );
}

export default function YellowPagesSettings(props: {
  ypConfigs: readonly YPConfig[];
  settings: Settings;
  readedTerms: { [url: string]: string };
  onReadTerms: (termsURL: string, url: string) => void;
  onChange(value: Settings): void;
}) {
  const update = (newSettings: Partial<Settings>) => {
    props.onChange({ ...props.settings, ...newSettings });
  };

  return (
    <div
      css={css`
        display: flex;
        gap: 64px 16px;
        flex-wrap: wrap;
      `}
    >
      {[
        {
          protocol: 'IPv4' as 'IPv4' | 'IPv6',
          ypConfigs: props.ypConfigs,
          usedHostForIPV4: undefined,
          value: props.settings.ipv4,
          onChange: (ipv4: EachYellowPagesSettings) => update({ ipv4 }),
        },
        {
          protocol: 'IPv6' as 'IPv4' | 'IPv6',
          ypConfigs: props.ypConfigs.filter((x) => x.supportIpv6),
          usedHostForIPV4: props.settings.ipv4.host,
          value: props.settings.ipv6,
          onChange: (ipv6: EachYellowPagesSettings) => update({ ipv6 }),
        },
      ].map(({ protocol, ypConfigs, usedHostForIPV4, value, onChange }) => (
        <EachYellowPagesSettingsView
          key={protocol}
          protocol={protocol}
          ypConfigs={ypConfigs}
          usedHostForIPV4={usedHostForIPV4}
          value={value}
          onChange={onChange}
          agreedTerms={props.settings.agreedTerms}
          readedTerms={props.readedTerms}
          onReadTerms={props.onReadTerms}
          onChangeAgreeTerms={(url, hash) =>
            update({
              agreedTerms: {
                ...props.settings.agreedTerms,
                [url]: hash ?? undefined!!,
              },
            })
          }
        />
      ))}
    </div>
  );
}
