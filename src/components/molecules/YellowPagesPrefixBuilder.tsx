import { css } from '@emotion/css';
import { Checkbox, Dropdown, ResponsiveMode, TextField } from '@fluentui/react';
import YPConfig, { YPConfigParams } from '../../entities/YPConfig';

export default function YellowPagesPrefixBuilder(props: {
  config: YPConfig | null;
  value: YPConfigParams;
  onChange(value: YPConfigParams): void;
}) {
  const supportedParams = props.config?.supportedParams ?? [];

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <TextField
        label="名前空間"
        disabled={!supportedParams.includes('namespace')}
        defaultValue={props.value.namespace}
        onBlur={(e) => {
          const namespace = e.target.value;
          if (namespace === props.value.namespace) {
            return;
          }
          props.onChange({ ...props.value, namespace });
        }}
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
        onChange={(_e, option) => {
          const portBandwidthCheck =
            option?.key as YPConfigParams['portBandwidthCheck'];
          if (portBandwidthCheck === props.value.portBandwidthCheck) {
            return;
          }
          props.onChange({ ...props.value, portBandwidthCheck });
        }}
      />
      <Checkbox
        label="リスナー数を隠す"
        disabled={!supportedParams.includes('hide_listeners')}
        checked={props.value.hideListeners}
        onChange={(_e, checked) =>
          props.onChange({ ...props.value, hideListeners: checked === true })
        }
      />
      <Checkbox
        label="ログを残さない"
        disabled={!supportedParams.includes('no_log')}
        checked={props.value.noLog}
        onChange={(_e, checked) =>
          props.onChange({ ...props.value, noLog: checked === true })
        }
      />
      <TextField
        label="アイコン"
        type="url"
        disabled={!supportedParams.includes('icon')}
        placeholder="https://"
        defaultValue={props.value.icon}
        onBlur={(e) => {
          const icon = e.target.value;
          if (icon === props.value.icon) {
            return;
          }
          props.onChange({ ...props.value, icon });
        }}
      />
    </div>
  );
}
