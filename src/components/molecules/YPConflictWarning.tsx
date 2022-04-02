import { css } from '@emotion/react';

export default function YPConflictWarning(): JSX.Element {
  return (
    <span
      css={css`
        background-color: white;
        border: 1px solid gray;
        padding: 4px;
        margin-right: 8px;
        position: absolute;
        font-weight: bold;
        z-index: 1;
      `}
    >
      IPv4 と同じ YP を指定すると
      <span
        css={css`
          white-space: nowrap;
        `}
      >
        チャンネル
      </span>
      を掲載できません
    </span>
  );
}
