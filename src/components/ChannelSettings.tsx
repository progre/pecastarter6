import { css } from '@emotion/react';
import { Dropdown, ResponsiveMode, TextField } from '@fluentui/react';
import { DOMAttributes, useState } from 'react';
import {
  ChannelContent,
  ChannelSettings as Settings,
} from '../entities/Settings';
import HistoryTextField from './molecules/HistoryTextField';

function History(props: {
  history: readonly ChannelContent[];
  onChange(value: ChannelContent): void;
}): JSX.Element {
  return (
    <Dropdown
      css={css`
        margin-right: 24px;
        display: flex;
        > div {
          margin-left: 8px;
          flex-grow: 1;
        }
      `}
      label="履歴"
      responsiveMode={ResponsiveMode.large}
      options={props.history.map((x, i) => ({
        key: `${x.genre} - ${x.desc}`,
        data: x,
        text: `${x.genre} - ${x.desc}`,
      }))}
      selectedKey={null}
      onChange={(e, option, _i) => props.onChange(option!!.data!!)}
    />
  );
}

function ChannelContentView(props: {
  channelContent: ChannelContent;
  onChange(value: Partial<ChannelContent>): void;
  onBlur: DOMAttributes<HTMLDivElement>['onBlur'];
}): JSX.Element {
  return (
    <div
      css={css`
        display: flex;
        flex-direction: row;
        gap: 8px;
      `}
      onBlur={props.onBlur}
    >
      <TextField
        label="ジャンル"
        value={props.channelContent.genre}
        onChange={(_e, genre) => props.onChange({ genre })}
      />
      <TextField
        css={css`
          flex-grow: 1;
        `}
        label="概要"
        value={props.channelContent.desc}
        onChange={(_e, desc) => props.onChange({ desc })}
      />
    </div>
  );
}

export default function ChannelSettings(props: {
  settings: Settings;
  onChange(value: Settings): void;
}) {
  const [channelContent, setChannelContent] = useState({
    genre: props.settings.genre,
    desc: props.settings.desc,
  });

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <ChannelContentView
        channelContent={channelContent}
        onChange={(newValue) => {
          setChannelContent((channelContent) => ({
            ...channelContent,
            ...newValue,
          }));
        }}
        onBlur={() => {
          props.onChange({
            ...props.settings,
            genre: channelContent.genre,
            desc: channelContent.desc,
          });
        }}
      />
      <History
        history={props.settings.channelContentHistory}
        onChange={(newChannelContent) => {
          setChannelContent(newChannelContent);
          props.onChange({
            ...props.settings,
            genre: newChannelContent.genre,
            desc: newChannelContent.desc,
          });
        }}
      />
      <div
        css={css`
          margin-top: 24px;
        `}
      >
        <HistoryTextField
          label="コメント"
          value={props.settings.comment[0]}
          history={props.settings.comment
            .slice(1)
            .filter((x) => x.trim() !== '')}
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
    </div>
  );
}
