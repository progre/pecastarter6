import { css } from '@emotion/react';
import {
  DefaultButton,
  IconButton,
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
import { invoke } from '@tauri-apps/api';
import { useCallback, useRef, useState } from 'react';
import { GeneralSettings as Settings } from '../entities/Settings';
import updatedHistory from '../utils/updatedHistory';
import HistoryTextField from './molecules/HistoryTextField';

type State = Settings & { workingChannelName: string };

function CopyableTextField(props: ITextFieldProps) {
  const ref = useRef<ITooltipHost>();
  return (
    <div
      css={css`
        display: flex;
        align-items: end;
      `}
    >
      <TextField
        css={css`
          flex-grow: 1;
        `}
        styles={{
          fieldGroup: {
            borderRight: 'none',
            borderTopRightRadius: 0,
            borderBottomRightRadius: 0,
          },
        }}
        {...props}
      />
      <TooltipHost
        content="Copied"
        overflowMode={TooltipOverflowMode.Self}
        componentRef={ref as IRefObject<ITooltipHost>}
      >
        <DefaultButton
          css={css`
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
          `}
          text="Copy"
          iconProps={{ iconName: 'copy' }}
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
      css={css`
        display: flex;
        align-items: end;
      `}
    >
      <SpinButton
        label="RTMP TCP ポート番号 (0 で自動)"
        style={{ width: '0' }}
        css={css`
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
        onChange={(_e, newValue) => setValue(Number(newValue))}
        onBlur={() => props.onChange(value)}
      />
      <DefaultButton
        css={css`
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
                (async (ev, item) => {
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
  const [state, setState] = useState({
    ...props.settings,
    workingChannelName: props.settings.channelName[0],
  });

  const update = (newState: Partial<State>) => {
    setState((state) => ({ ...state, ...newState }));
  };

  const serverForObs = `rtmp://localhost${
    state.rtmpListenPort === 1935 ? '' : `:${state.rtmpListenPort}`
  }/live/livestream`;

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <HistoryTextField
        label="チャンネル名"
        required
        history={state.channelName.filter((x) => x.trim() !== '')}
        value={state.workingChannelName}
        onChange={(value) =>
          setState((state) => ({ ...state, ...{ workingChannelName: value } }))
        }
        onBlurCapture={(_ev) => {
          const generalSettings = {
            ...state,
            channelName: updatedHistory(
              state.workingChannelName,
              state.channelName,
              5
            ),
          };
          props.onChange(generalSettings);
        }}
      />
      <SpinButton
        label="RTMP 待ち受け TCP ポート番号"
        style={{ width: '0' }}
        styles={{ input: { textAlign: 'end' } }}
        max={65535}
        min={1}
        css={css`
          margin-top: 24px;
        `}
        value={String(state.rtmpListenPort)}
        onChange={(_e, newValue) =>
          update({ rtmpListenPort: Number(newValue) })
        }
      />
      <CopyableTextField
        label="OBS にカスタムサーバーとして設定する値"
        readOnly
        value={serverForObs}
      />
      <Separator />
      <Text variant="large">PeerCastStation</Text>
      <SpinButton
        label="データ通信 TCP ポート番号"
        style={{ width: '0' }}
        styles={{ input: { textAlign: 'end' } }}
        max={65535}
        min={1}
        value={String(state.peerCastPort)}
        onChange={(_e, newValue) => update({ peerCastPort: Number(newValue) })}
      />
      <PeerCastRtmpTcpPort
        value={state.peerCastRtmpPort}
        onChange={(peerCastRtmpPort) => {
          update({ peerCastRtmpPort });
          const generalSettings = {
            ...state,
            peerCastRtmpPort,
            channelName: updatedHistory(
              state.workingChannelName,
              state.channelName,
              5
            ),
          };
          props.onChange(generalSettings);
        }}
      />
    </div>
  );
}
