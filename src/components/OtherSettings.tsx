import { css } from '@emotion/react';
import { Checkbox, DefaultButton, TextField } from '@fluentui/react';
import { dialog, invoke, shell } from '@tauri-apps/api';
import { useState } from 'react';
import { OtherSettings as Settings } from '../entities/Settings';

export default function OtherSettings(props: {
  settings: Settings;
  onChange(value: Settings): void;
}) {
  const [logOutputDirectory, setLogOutputDirectory] = useState(
    props.settings.logOutputDirectory
  );

  return (
    <div
      css={css`
        display: flex;
        flex-direction: column;
        gap: 8px;
      `}
    >
      <Checkbox
        label="配信ログを保存する"
        checked={props.settings.logEnabled}
        onChange={(_ev, logEnabled) =>
          props.onChange({ ...props.settings, logEnabled: logEnabled === true })
        }
      />
      <div
        css={css`
          display: flex;
          align-items: end;
        `}
      >
        <TextField
          css={css`
            flex-grow: 1;
          `}
          styles={{
            fieldGroup: {
              borderRight: 'none',
              borderTopRightRadius: 0,
              borderBottomRightRadius: 0,
            },
          }}
          label="ログの出力先"
          disabled={!props.settings.logEnabled}
          value={logOutputDirectory}
          onChange={(_ev, newValue) => setLogOutputDirectory(newValue!!)}
          onBlur={(ev) =>
            props.onChange({
              ...props.settings,
              logOutputDirectory: ev.target.value,
            })
          }
        />
        <DefaultButton
          css={css`
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
            min-width: 0;
          `}
          iconProps={{ iconName: 'folderopen' }}
          disabled={!props.settings.logEnabled}
          onClick={async () => {
            const logOutputDirectory = (await dialog.open({
              defaultPath: props.settings.logOutputDirectory,
              directory: true,
            })) as string | null;
            if (logOutputDirectory == null) {
              return;
            }
            setLogOutputDirectory(logOutputDirectory);
            props.onChange({ ...props.settings, logOutputDirectory });
          }}
        />
      </div>
      <div
        css={css`
          margin-top: 4ex;
        `}
      >
        <DefaultButton
          iconProps={{ iconName: 'folder' }}
          onClick={async () => {
            invoke('open_app_dir');
          }}
        >
          設定ファイルの場所を開く
        </DefaultButton>
      </div>
    </div>
  );
}
