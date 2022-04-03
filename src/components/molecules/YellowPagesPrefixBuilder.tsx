import { css } from '@emotion/react';
import { Checkbox, Dropdown, ResponsiveMode, TextField } from '@fluentui/react';
import { useCallback } from 'react';
import YPConfig, { YPConfigParams } from '../../entities/YPConfig';

export default function YellowPagesPrefixBuilder(props: {
  config: YPConfig | null;
  value: YPConfigParams;
  onChange(value: YPConfigParams): void;
}) {
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
        disabled={!supportedParams.includes('namespace')}
        defaultValue={props.value.namespace}
        onBlur={(e) => update({ namespace: e.target.value })}
      />
      <Dropdown
        label="ポートチェック"
        disabled={!supportedParams.includes('port_bandwidth_check')}
        selectedKey={props.value.portBandwidthCheck}
        responsiveMode={ResponsiveMode.large}
        options={[
          'なし',
          'ポートチェック',
          'ポート&帯域チェック',
          'ポート&高速帯域チェック',
        ].map((text, key) => ({ key, text }))}
        onChange={(_e, option) =>
          update({
            portBandwidthCheck:
              option?.key as YPConfigParams['portBandwidthCheck'],
          })
        }
      />
      <Checkbox
        label="リスナー数を隠す"
        disabled={!supportedParams.includes('hide_listeners')}
        checked={props.value.hideListeners}
        onChange={(_e, checked) => update({ hideListeners: checked })}
      />
      <Checkbox
        label="ログを残さない"
        disabled={!supportedParams.includes('no_log')}
        checked={props.value.noLog}
        onChange={(_e, checked) => update({ noLog: checked })}
      />
      <TextField
        label="アイコン"
        type="url"
        disabled={!supportedParams.includes('icon')}
        placeholder="https://"
        defaultValue={props.value.icon}
        onBlur={(e) => update({ icon: e.target.value })}
      />
    </div>
  );
}
