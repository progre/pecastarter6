import { css } from '@emotion/css';
import {
  DefaultButton,
  Dropdown,
  IDropdown,
  ResponsiveMode,
  Text,
  TextField,
} from '@fluentui/react';
import { DOMAttributes, useRef, useState } from 'react';
import {
  ChannelContent,
  ChannelSettings as Settings,
} from '../entities/Settings';
import HistoryTextField from './molecules/HistoryTextField';
import ShowMore from './molecules/ShowMore';

function History(props: {
  currentGenre: string;
  currentDesc: string;
  history: readonly ChannelContent[];
  onChange(value: ChannelContent): void;
}): JSX.Element {
  const componentRef = useRef<IDropdown>(null);
  const [isOpen, setOpen] = useState(false);
  const limit = 5;
  const [extended, setExtended] = useState(false);
  return (
    <div
      className={css`
        display: flex;
      `}
    >
      <Dropdown
        componentRef={componentRef}
        onFocus={() => setOpen(true)}
        onBlur={() => setOpen(false)}
        className={css`
          flex-grow: 1;
          opacity: 0;
          pointer-events: none;
        `}
        styles={{ dropdown: { width: 0 } }}
        responsiveMode={ResponsiveMode.large}
        onRenderList={(renderProps, defaultRender) => (
          <>
            {defaultRender!!(renderProps)}
            <ShowMore
              hidden={props.history.length < limit || extended}
              onClick={() => setExtended(true)}
            />
          </>
        )}
        options={props.history
          .slice(0, extended ? props.history.length : limit)
          .map((x, _i) => ({
            key: `${x.genre} - ${x.desc}`,
            data: x,
            text: `${x.genre} - ${x.desc}`,
          }))}
        selectedKey={`${props.currentGenre} - ${props.currentDesc}`}
        onDismiss={() => setExtended(false)}
        onChange={(_e, option, _i) => props.onChange(option!!.data!!)}
      />
      <DefaultButton
        className={css`
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
  onSelect(value: ChannelContent): void;
  onBlur: DOMAttributes<HTMLDivElement>['onBlur'];
}): JSX.Element {
  return (
    <div
      className={css`
        position: relative;
      `}
    >
      <div
        className={css`
          width: 100%;
          position: absolute;
          margin-top: 26px;
        `}
      >
        <History
          currentGenre={props.channelContent.genre}
          currentDesc={props.channelContent.desc}
          history={props.history}
          onChange={(newChannelContent) =>
            props.onSelect({
              genre: newChannelContent.genre,
              desc: newChannelContent.desc,
            })
          }
        />
      </div>
      <div
        className={css`
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
          className={css`
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
  contactStatus: { title: string; resCount: number };
  onChange(value: Settings): void;
}) {
  const [channelContent, setChannelContent] = useState({
    genre: props.settings.genre,
    desc: props.settings.desc,
  });

  return (
    <div
      className={css`
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
        onSelect={(newChannelContent) => {
          setChannelContent(newChannelContent);
          if (
            newChannelContent.genre === props.settings.genre &&
            newChannelContent.desc === props.settings.desc
          ) {
            return;
          }
          props.onChange({ ...props.settings, ...newChannelContent });
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
        className={css`
          margin-top: 24px;
          display: flex;
          flex-direction: column;
          gap: 8px;
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
        <div>
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
          <div
            className={css`
              width: 100%;
              display: ${props.contactStatus.title === '' ? 'none' : 'flex'};
              margin-top: 1ex;
            `}
          >
            <Text nowrap variant="small">
              <a target="_blank" href={props.settings.contactUrl[0]}>
                {props.contactStatus.title}
              </a>
            </Text>
            <Text
              className={css`
                margin-left: 0.25em;
                ${props.contactStatus.resCount === 0 ? 'display: none' : ''}
              `}
              variant="small"
            >
              ({props.contactStatus.resCount})
            </Text>
          </div>
        </div>
      </div>
    </div>
  );
}
