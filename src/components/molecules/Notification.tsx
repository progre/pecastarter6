import { css } from '@emotion/css';
import { MessageBar, MessageBarType } from '@fluentui/react';

export default function Notification(props: {
  level: string;
  message: string;

  onClickClose(): void;
}) {
  return (
    <div
      className={css`
        position: absolute;
        bottom: 0;
        user-select: none;
        width: 100%;
        z-index: 1;
      `}
    >
      <MessageBar
        messageBarType={
          {
            fatal: MessageBarType.error,
            error: MessageBarType.severeWarning,
            warn: MessageBarType.warning,
          }[props.level] ?? MessageBarType.info
        }
        isMultiline={true}
        dismissButtonAriaLabel="Close"
        onDismiss={() => props.onClickClose()}
        styles={{
          content: {
            display: 'flex',
            alignItems: 'center',
          },
          text: {
            display: 'flex',
            alignItems: 'center',
          },
          dismissSingleLine: {
            display: 'flex',
            alignItems: 'center',
          },
        }}
      >
        {props.message}
      </MessageBar>
    </div>
  );
}
