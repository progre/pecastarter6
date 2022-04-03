import { css } from '@emotion/react';
import {
  DefaultButton,
  IRefObject,
  ITooltipHost,
  SpinButton,
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

export default function GeneralSettings(props: { defaultSettings: Settings }) {
  const [state, setState] = useState({
    ...props.defaultSettings,
    workingChannelName: props.defaultSettings.channelName[0],
  });

  const onBlur = useCallback(() => {
    const generalSettings = {
      ...state,
      channelName: updatedHistory(
        state.workingChannelName,
        state.channelName,
        5
      ),
    };
    setState(generalSettings);
    invoke('set_general_settings', { generalSettings });
  }, [state]);

  const update = (newState: Partial<State>) => {
    setState((state) => ({ ...state, ...newState }));
  };

  const id = `_${(Math.random() * Number.MAX_SAFE_INTEGER) | 0}`;

  const serverForObs = `rtmp://localhost${
    state.rtmpListenPort === 1935 ? '' : `:${state.rtmpListenPort}`
  }/live/livestream`;

  const ref = useRef<ITooltipHost>();

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
      onBlur={onBlur}
    >
      <HistoryTextField
        label="チャンネル名"
        required
        history={state.channelName.filter((x) => x.trim() !== '')}
        value={state.workingChannelName}
        onChange={(value) => update({ workingChannelName: value })}
      />
      <SpinButton
        label="PeerCastStation の通信用 TCP ポート番号"
        max={65535}
        min={1}
        value={String(state.peerCastPort)}
        onChange={(_e, newValue) => update({ peerCastPort: Number(newValue) })}
      />
      <SpinButton
        label="PeCa Starter の RTMP 待ち受け TCP ポート番号"
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
          label="OBS にカスタムサーバーとして設定する値"
          readOnly
          value={serverForObs}
        />
        <TooltipHost
          content="Copied"
          overflowMode={TooltipOverflowMode.Self}
          componentRef={ref as IRefObject<ITooltipHost>}
        >
          <DefaultButton
            text="Copy"
            iconProps={{ iconName: 'copy' }}
            onClick={() => {
              navigator.clipboard.writeText(serverForObs);
              ref.current!!.show();
            }}
          />
        </TooltipHost>
      </div>
    </div>
  );
}
