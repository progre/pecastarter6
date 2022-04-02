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

function EachYellowPagesSettingsView(props: {
  protocol: 'IPv4' | 'IPv6';
  ypConfigs: readonly YPConfig[];
  usedHostForIPV4?: string;
  agreedTerms: { [url: string]: string };
  value: EachYellowPagesSettings;
  onChange(value: EachYellowPagesSettings): void;
  onChangeAgreeTerms(url: string, hash: string | null): void;
}): JSX.Element {
  const id = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;
  const currentYPConfig = props.ypConfigs.find(
    (x) => x.host === props.value.host
  );
  const conflict =
    currentYPConfig != null && currentYPConfig.host === props.usedHostForIPV4;
  const [readedTerms, setReadedTerms] = useState<string | null>(
    props.agreedTerms[currentYPConfig?.termsURL ?? ''] ?? null
  );
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
        <YPSelect
          label={`${props.protocol} 掲載 YP`}
          ypConfigs={props.ypConfigs}
          usedHostForIPV4={props.usedHostForIPV4}
          conflict={conflict}
          host={props.value.host}
          onChange={(host) => {
            setReadedTerms(null);
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
          setReadedTerms(termsHash);
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
  onChange(value: Settings): void;
  onBlur(): void;
}) {
  const update = (newSettings: Partial<Settings>) => {
    props.onChange({ ...props.settings, ...newSettings });
  };

  return (
    <div
      css={css`
        display: flex;
        gap: 16px;
        flex-wrap: wrap;
      `}
      onBlur={props.onBlur}
    >
      <EachYellowPagesSettingsView
        protocol="IPv4"
        ypConfigs={props.ypConfigs}
        agreedTerms={props.settings.agreedTerms}
        value={props.settings.ipv4}
        onChange={(ipv4) => update({ ipv4 })}
        onChangeAgreeTerms={(url, hash) =>
          update({
            agreedTerms: {
              ...props.settings.agreedTerms,
              [url]: hash ?? undefined!!,
            },
          })
        }
      />
      <EachYellowPagesSettingsView
        protocol="IPv6"
        ypConfigs={props.ypConfigs.filter((x) => x.supportIpv6)}
        usedHostForIPV4={props.settings.ipv4.host}
        agreedTerms={props.settings.agreedTerms}
        value={props.settings.ipv6}
        onChange={(ipv6) => update({ ipv6 })}
        onChangeAgreeTerms={(url, hash) =>
          update({
            agreedTerms: {
              ...props.settings.agreedTerms,
              [url]: hash ?? undefined!!,
            },
          })
        }
      />
    </div>
  );
}
