import { css } from '@emotion/react';
import { useCallback } from 'react';
import YPConfig, { YPConfigParams } from '../../entities/YPConfig';
import TextField from './TextField';

export default function YellowPagesPrefixBuilder(props: {
  config: YPConfig | null;
  value: YPConfigParams;
  onChange(value: YPConfigParams): void;
}) {
  const portCheckId = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;
  const listenersInvisibilityId = `_${
    (Math.random() * Number.MAX_SAFE_INTEGER) | 0
  }`;
  const noLogId = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;
  const supportedParams = props.config?.supportedParams ?? [];

  const update = useCallback(
    (partial: Partial<YPConfigParams>) => {
      props.onChange({ ...props.value, ...partial });
    },
    [props.value, props.onChange]
  );

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <TextField
        label="名前空間"
        type="text"
        disabled={!supportedParams.includes('namespace')}
        value={props.value.namespace}
        onChangeValue={(value) => update({ namespace: value })}
      />
      <div
        css={css`
          display: flex;
          flex-wrap: wrap;
        `}
      >
        <label
          htmlFor={portCheckId}
          css={css`
            padding-right: 0.5em;
            color: ${!supportedParams.includes('port_bandwidth_check')
              ? 'lightgray'
              : 'inherit'};
          `}
        >
          ポートチェック:
        </label>
        <select
          id={portCheckId}
          disabled={!supportedParams.includes('port_bandwidth_check')}
          value={props.value.portBandwidthCheck}
          css={css`
            flex-grow: 1;
            color: ${!supportedParams.includes('port_bandwidth_check')
              ? 'lightgray'
              : 'inherit'};
          `}
          onChange={(e) =>
            update({
              portBandwidthCheck: Number(
                e.target.value
              ) as YPConfigParams['portBandwidthCheck'],
            })
          }
        >
          {[
            'なし',
            'ポートチェック',
            'ポート&帯域チェック',
            'ポート&高速帯域チェック',
          ].map((x, i) => (
            <option key={i} value={i}>
              {x}
            </option>
          ))}
        </select>
      </div>
      <div>
        <input
          id={listenersInvisibilityId}
          type="checkbox"
          disabled={!supportedParams.includes('hide_listeners')}
          checked={props.value.hideListeners}
          onChange={(e) => update({ hideListeners: e.target.checked })}
        />
        <label
          htmlFor={listenersInvisibilityId}
          css={css`
            padding-left: 0.25em;
            color: ${!supportedParams.includes('hide_listeners')
              ? 'lightgray'
              : 'inherit'};
          `}
        >
          リスナー数を隠す
        </label>
      </div>
      <div>
        <input
          id={noLogId}
          type="checkbox"
          disabled={!supportedParams.includes('no_log')}
          checked={props.value.noLog}
          onChange={(e) => update({ noLog: e.target.checked })}
        />
        <label
          htmlFor={noLogId}
          css={css`
            padding-left: 0.25em;
            color: ${!supportedParams.includes('no_log')
              ? 'lightgray'
              : 'inherit'};
          `}
        >
          ログを残さない
        </label>
      </div>
      <TextField
        label="アイコン"
        type="url"
        disabled={!supportedParams.includes('icon')}
        placeholder="https://"
        value={props.value.icon}
        onChangeValue={(value) => update({ icon: value })}
      />
    </div>
  );
}
