import { css } from '@emotion/css';

export default function YPConflictWarning(): JSX.Element {
  return (
    <span
      className={css`
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
        className={css`
          white-space: nowrap;
        `}
      >
        チャンネル
      </span>
      を掲載できません
    </span>
  );
}
