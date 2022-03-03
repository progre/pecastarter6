import { css } from '@emotion/react';
import { invoke } from '@tauri-apps/api';
import { useCallback, useState } from 'react';
import { ChannelSettings as Settings } from '../entities/Settings';
import updatedHistory from '../utils/updatedHistory';
import TextField from './molecules/TextField';

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
      <TextField
        label="ジャンル"
        type="text"
        value={state.workingGenre}
        history={state.genre.filter((x) => x.trim() !== '')}
        onChangeValue={(value) => update({ workingGenre: value })}
      />
      <TextField
        label="概要"
        type="text"
        value={state.workingDesc}
        history={state.desc.filter((x) => x.trim() !== '')}
        onChangeValue={(value) => update({ workingDesc: value })}
      />
      <TextField
        label="コメント"
        type="text"
        value={state.workingComment}
        history={state.comment.filter((x) => x.trim() !== '')}
        onChangeValue={(value) => update({ workingComment: value })}
      />
      <TextField
        label="コンタクト URL"
        type="url"
        placeholder="https://"
        value={state.workingContactUrl}
        history={state.contactUrl.filter((x) => x.trim() !== '')}
        onChangeValue={(value) => update({ workingContactUrl: value })}
      />
    </div>
  );
}
