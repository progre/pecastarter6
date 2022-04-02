import { css } from '@emotion/react';
import { TextField } from '@fluentui/react';
import { invoke } from '@tauri-apps/api';
import { useCallback, useState } from 'react';
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

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
      onBlur={onBlur}
    >
      <TextField
        label="PeerCastStation の通信用 TCP ポート番号"
        type="number"
        max={65535}
        min={1}
        required
        value={String(state.peerCastPort)}
        onChange={(_e, newValue) => update({ peerCastPort: Number(newValue) })}
      />
      <HistoryTextField
        label="チャンネル名"
        required
        history={state.channelName.filter((x) => x.trim() !== '')}
        value={state.workingChannelName}
        onChange={(value) => update({ workingChannelName: value })}
      />
      <TextField
        label="PeerCastStation の通信用 TCP ポート番号"
        type="number"
        max={65535}
        min={1}
        required
        value={String(state.rtmpListenPort)}
        onChange={(_e, newValue) =>
          update({ rtmpListenPort: Number(newValue) })
        }
      />
    </div>
  );
}
