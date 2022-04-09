import { css } from '@emotion/react';
import { ChannelSettings as Settings } from '../entities/Settings';
import updatedHistory from '../utils/updatedHistory';
import HistoryTextField from './molecules/HistoryTextField';

function ChannelContent(props: {
  settings: Settings;
  onChange(value: Settings): void;
}): JSX.Element {
  return (
    <>
      <HistoryTextField
        label="ジャンル"
        value={props.settings.genre[0]}
        history={props.settings.genre.filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            genre: updatedHistory(value, props.settings.genre, 20),
          })
        }
      />
      <HistoryTextField
        label="概要"
        value={props.settings.desc[0]}
        history={props.settings.desc.filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            desc: updatedHistory(value, props.settings.desc, 20),
          })
        }
      />
    </>
  );
}

export default function ChannelSettings(props: {
  settings: Settings;
  onChange(value: Settings): void;
}) {
  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <ChannelContent settings={props.settings} onChange={props.onChange} />
      <HistoryTextField
        label="コメント"
        value={props.settings.comment[0]}
        history={props.settings.comment.filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            comment: updatedHistory(value, props.settings.comment, 20),
          })
        }
      />
      <HistoryTextField
        label="コンタクト URL"
        placeholder="https://"
        value={props.settings.contactUrl[0]}
        history={props.settings.contactUrl.filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            contactUrl: updatedHistory(value, props.settings.contactUrl, 20),
          })
        }
      />
    </div>
  );
}
