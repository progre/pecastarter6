import { css } from '@emotion/react';
import { DefaultButton, Separator } from '@fluentui/react';

export default function ShowMore(props: {
  hidden: boolean;
  onClick(): void;
}): JSX.Element {
  return (
    <div
      css={css`
        ${props.hidden ? 'display: none;' : 'display: flex;'}
        flex-direction: column;
      `}
    >
      <Separator
        css={css`
          margin: 0 8px 1px;
          height: 8px;
        `}
      />
      <DefaultButton
        css={css`
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
