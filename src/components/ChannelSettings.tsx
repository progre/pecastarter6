import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api';
import { useCallback, useState } from 'react';
import { ChannelSettings as Settings } from '../entities/Settings';
import updatedHistory from '../utils/updatedHistory';
import HistoryTextField from './molecules/HistoryTextField';

type State = Settings & {
  workingGenre: string;
  workingDesc: string;
  workingComment: string;
  workingContactUrl: string;
};

export default function ChannelSettings(props: { defaultSettings: Settings }) {
  const [state, setState] = useState({
    ...props.defaultSettings,
    workingGenre: props.defaultSettings.genre[0],
    workingDesc: props.defaultSettings.desc[0],
    workingComment: props.defaultSettings.comment[0],
    workingContactUrl: props.defaultSettings.contactUrl[0],
  });

  const onBlur = useCallback(() => {
    const channelSettings = {
      ...state,
      genre: updatedHistory(state.workingGenre, state.genre, 20),
      desc: updatedHistory(state.workingDesc, state.desc, 20),
      comment: updatedHistory(state.workingComment, state.comment, 20),
      contactUrl: updatedHistory(state.workingContactUrl, state.contactUrl, 20),
    };
    setState(channelSettings);
    invoke('set_channel_settings', { channelSettings });
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
      <HistoryTextField
        label="ジャンル"
        value={state.workingGenre}
        history={state.genre.filter((x) => x.trim() !== '')}
        onChange={(value) => update({ workingGenre: value })}
      />
      <HistoryTextField
        label="概要"
        value={state.workingDesc}
        history={state.desc.filter((x) => x.trim() !== '')}
        onChange={(value) => update({ workingDesc: value })}
      />
      <HistoryTextField
        label="コメント"
        value={state.workingComment}
        history={state.comment.filter((x) => x.trim() !== '')}
        onChange={(value) => update({ workingComment: value })}
      />
      <HistoryTextField
        label="コンタクト URL"
        placeholder="https://"
        value={state.workingContactUrl}
        history={state.contactUrl.filter((x) => x.trim() !== '')}
        onChange={(value) => update({ workingContactUrl: value })}
      />
    </div>
  );
}
