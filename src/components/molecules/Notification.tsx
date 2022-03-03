import { css } from '@emotion/react';

export default function Notification(props: {
  level: string;
  message: string;

  onClickClose(): void;
}) {
  return (
    <div
      css={css`
        position: absolute;
        bottom: 0;
        pointer-events: none;
        user-select: none;
        width: 100%;
      `}
    >
      <div
        css={css`
          display: flex;
          padding: 8px;
          background-color: #f9f9f9;
        `}
      >
        <div
          css={css`
            width: 100%;
            display: flex;
          `}
        >
          <div>{props.message}</div>
        </div>
        <button
          css={css`
            background: none;
            border: none;
            cursor: pointer;
            pointer-events: auto;
          `}
          onClick={() => props.onClickClose()}
        >
          ×
        </button>
      </div>
    </div>
  );
}
