import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api';
import { useCallback, useState } from 'react';
import { GeneralSettings as Settings } from '../entities/Settings';
import updatedHistory from '../utils/updatedHistory';
import TextField from './molecules/TextField';

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

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 24px;
      `}
      onBlur={onBlur}
    >
      <TextField
        label="PeerCastStation の通信用 TCP ポート番号"
        type="number"
        max={65535}
        min={1}
        required
        value={state.peerCastPort}
        fitContent
        onChangeValueAsNumber={(value) => update({ peerCastPort: value })}
      />
      <TextField
        label="チャンネル名"
        type="text"
        required
        value={state.workingChannelName}
        history={state.channelName.filter((x) => x.trim() !== '')}
        onChangeValue={(value) => update({ workingChannelName: value })}
      />
      <TextField
        label="PeCa Starter の RTMP 待ち受け TCP ポート番号"
        type="number"
        max={65535}
        min={1}
        required
        value={state.rtmpListenPort}
        fitContent
        onChangeValueAsNumber={(value) => update({ rtmpListenPort: value })}
      />
    </div>
  );
}
