import { css } from '@emotion/react';
import {
  DefaultButton,
  Dropdown,
  IDropdown,
  ResponsiveMode,
  TextField,
} from '@fluentui/react';
import { DOMAttributes, useRef, useState } from 'react';
import {
  ChannelContent,
  ChannelSettings as Settings,
} from '../entities/Settings';
import HistoryTextField from './molecules/HistoryTextField';

function History(props: {
  history: readonly ChannelContent[];
  onChange(value: ChannelContent): void;
}): JSX.Element {
  const componentRef = useRef<IDropdown>(null);
  const [isOpen, setOpen] = useState(false);
  return (
    <div
      css={css`
        display: flex;
      `}
    >
      <Dropdown
        componentRef={componentRef}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        css={css`
          flex-grow: 1;
          opacity: 0;
          pointer-events: none;
        `}
        responsiveMode={ResponsiveMode.large}
        options={props.history.map((x, _i) => ({
          key: `${x.genre} - ${x.desc}`,
          data: x,
          text: `${x.genre} - ${x.desc}`,
        }))}
        selectedKey={null}
        onChange={(_e, option, _i) => props.onChange(option!!.data!!)}
      />
      <DefaultButton
        css={css`
          min-width: 32px;
          padding-left: 0;
          padding-right: 0;
        `}
        iconProps={{
          iconName: 'chevrondown',
          styles: { root: { fontSize: 12 } },
        }}
        onClick={() => componentRef.current!.focus(!isOpen)}
      />
    </div>
  );
}

function ChannelContentView(props: {
  history: readonly ChannelContent[];
  channelContent: ChannelContent;
  onChange(value: Partial<ChannelContent>): void;
  onBlur: DOMAttributes<HTMLDivElement>['onBlur'];
}): JSX.Element {
  return (
    <div
      css={css`
        position: relative;
      `}
    >
      <div
        css={css`
          width: 100%;
          position: absolute;
          margin-top: 31.5px;
        `}
      >
        <History
          history={props.history}
          onChange={(newChannelContent) =>
            props.onChange({
              genre: newChannelContent.genre,
              desc: newChannelContent.desc,
            })
          }
        />
      </div>
      <div
        css={css`
          margin-right: 32px;
          padding-right: 8px;
          display: flex;
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
        history={props.settings.channelContentHistory}
        channelContent={channelContent}
        onChange={(newValue) => {
          setChannelContent((channelContent) => ({
            ...channelContent,
            ...newValue,
          }));
        }}
        onBlur={() => {
          if (
            channelContent.genre === props.settings.genre &&
            channelContent.desc === props.settings.desc
          ) {
            return;
          }
          props.onChange({ ...props.settings, ...channelContent });
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
