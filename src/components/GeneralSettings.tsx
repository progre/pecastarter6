import { css } from '@emotion/css';
import {
  DefaultButton,
  IRefObject,
  ITextFieldProps,
  ITooltipHost,
  Separator,
  SpinButton,
  Text,
  TextField,
  TooltipHost,
  TooltipOverflowMode,
} from '@fluentui/react';
import { invoke } from '@tauri-apps/api/core';
import { useRef, useState } from 'react';
import { GeneralSettings as Settings } from '../entities/Settings';
import HistoryTextField from './molecules/HistoryTextField';

function CopyableTextField(props: ITextFieldProps) {
  const ref = useRef<ITooltipHost>();
  return (
    <div
      className={css`
        display: flex;
        align-items: end;
      `}
    >
      <TextField
        className={css`
          flex-grow: 1;

          > div > div > input {
            background-color: rgb(243, 242, 241);
            color: rgb(161, 159, 157);
          }
        `}
        styles={{
          fieldGroup: {
            borderRight: 'none',
            borderTopRightRadius: 0,
            borderBottomRightRadius: 0,
          },
        }}
        readOnly
        {...props}
      />
      <TooltipHost
        content="Copied"
        overflowMode={TooltipOverflowMode.Self}
        componentRef={ref as IRefObject<ITooltipHost>}
      >
        <DefaultButton
          className={css`
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
          `}
          text="Copy"
          iconProps={{ iconName: 'clipboard' }}
          onClick={() => {
            navigator.clipboard.writeText(props.value!!);
            ref.current!!.show();
          }}
        />
      </TooltipHost>
    </div>
  );
}

function PeerCastRtmpTcpPort(props: {
  value: number;
  onChange(newValue: number): void;
}): JSX.Element {
  const [value, setValue] = useState(props.value);
  return (
    <div
      className={css`
        display: flex;
        align-items: end;
      `}
    >
      <SpinButton
        label="RTMP TCP ポート番号 (0 で自動)"
        style={{ width: 0 }}
        styles={{ input: { textAlign: 'end', textOverflow: 'clip' } }}
        className={css`
          z-index: 1;
          width: auto;
          > div:nth-of-type(2)::after {
            border-top-right-radius: 0;
            border-bottom-right-radius: 0;
          }
        `}
        max={65535}
        min={0}
        value={String(value)}
        onChange={(_e, newValue) => {
          setValue(Number(newValue));
          props.onChange(Number(newValue));
        }}
      />
      <DefaultButton
        className={css`
          border-left: none;
          border-top-left-radius: 0;
          border-bottom-left-radius: 0;
          min-width: 0;
          padding: 0 8px;
        `}
        menuProps={{
          items: [
            {
              key: 'emailMessage',
              text: '空きポートを探す',
              onClick: () => {
                (async (_ev, _item) => {
                  const newValue: number = await invoke('find_free_port');
                  setValue(newValue);
                  props.onChange(newValue);
                })();
              },
            },
          ],
        }}
        iconProps={{ iconName: 'search' }}
      />
    </div>
  );
}

export default function GeneralSettings(props: {
  settings: Settings;
  onChange(value: Settings): void;
}) {
  const serverForObs = `rtmp://localhost${props.settings.rtmpListenPort === 1935
    ? ''
    : `:${props.settings.rtmpListenPort}`
    }/live/livestream`;

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <HistoryTextField
        label="チャンネル名"
        required
        history={props.settings.channelName
          .slice(1)
          .filter((x) => x.trim() !== '')}
        value={props.settings.channelName[0]}
        onChange={(value) => {
          const newState = {
            ...props.settings,
            channelName: [value, ...props.settings.channelName.slice(1)],
          };
          props.onChange(newState);
        }}
      />
      <SpinButton
        label="RTMP 待ち受け TCP ポート番号"
        className={css`
          margin-top: 24px;
        `}
        style={{ width: 0 }}
        styles={{ input: { textAlign: 'end', textOverflow: 'clip' } }}
        max={65535}
        min={1}
        value={String(props.settings.rtmpListenPort)}
        onChange={(_ev, newValue) =>
          props.onChange({
            ...props.settings,
            rtmpListenPort: Number(newValue),
          })
        }
      />
      <CopyableTextField
        label="OBS にカスタムサーバーとして設定する値"
        value={serverForObs}
      />
      <Separator />
      <Text variant="large">PeerCastStation</Text>
      <SpinButton
        label="データ通信 TCP ポート番号"
        style={{ width: 0 }}
        styles={{ input: { textAlign: 'end', textOverflow: 'clip' } }}
        max={65535}
        min={1}
        value={String(props.settings.peerCastPort)}
        onChange={(_ev, newValue) =>
          props.onChange({
            ...props.settings,
            peerCastPort: Number(newValue),
          })
        }
      />
      <PeerCastRtmpTcpPort
        value={props.settings.peerCastRtmpPort}
        onChange={(peerCastRtmpPort) =>
          props.onChange({ ...props.settings, peerCastRtmpPort })
        }
      />
    </div>
  );
}
