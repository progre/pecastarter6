import { css } from '@emotion/react';
import { ChannelSettings as Settings } from '../entities/Settings';
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
        history={props.settings.genre.slice(1).filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            genre: [value, ...props.settings.genre.slice(1)],
          })
        }
      />
      <HistoryTextField
        label="概要"
        value={props.settings.desc[0]}
        history={props.settings.desc.slice(1).filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            desc: [value, ...props.settings.desc.slice(1)],
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
        history={props.settings.comment.slice(1).filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            comment: [value, ...props.settings.comment.slice(1)],
          })
        }
      />
      <HistoryTextField
        label="コンタクト URL"
        placeholder="https://"
        value={props.settings.contactUrl[0]}
        history={props.settings.contactUrl
          .slice(1)
          .filter((x) => x.trim() !== '')}
        onChange={(value) =>
          props.onChange({
            ...props.settings,
            contactUrl: [value, ...props.settings.contactUrl.slice(1)],
          })
        }
      />
    </div>
  );
}
