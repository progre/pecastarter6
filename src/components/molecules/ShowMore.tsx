import { css } from '@emotion/css';
import { DefaultButton, Separator } from '@fluentui/react';

export default function ShowMore(props: {
  hidden: boolean;
  onClick(): void;
}): JSX.Element {
  return (
    <div
      className={css`
        ${props.hidden ? 'display: none;' : 'display: flex;'}
        flex-direction: column;
      `}
    >
      <Separator
        className={css`
          margin: 0 8px 1px;
          height: 8px;
        `}
      />
      <DefaultButton
        className={css`
          height: 36px;
          border: none;
          padding: 0 4px;
          text-align: start;
        `}
        styles={{
          label: {
            fontWeight: 'initial',
          },
        }}
        onClick={props.onClick}
      >
        More...
      </DefaultButton>
    </div>
  );
}
