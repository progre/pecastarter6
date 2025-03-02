import { css } from '@emotion/css';
import { Checkbox, DefaultButton, TextField } from '@fluentui/react';
import { invoke } from '@tauri-apps/api/core';
import * as dialog from "@tauri-apps/plugin-dialog"
import { useState } from 'react';
import { LiteralUnion } from 'type-fest';
import { OtherSettings as Settings } from '../entities/Settings';

export default function OtherSettings(props: {
  // WTF: mac だとディレクトリ選択ダイアログが正常に動作しない
  platform: LiteralUnion<
    | 'linux'
    | 'darwin'
    | 'ios'
    | 'freebsd'
    | 'dragonfly'
    | 'netbsd'
    | 'openbsd'
    | 'solaris'
    | 'android'
    | 'win32',
    string
  >;
  version: string;
  settings: Settings;
  onChange(value: Settings): void;
}) {
  const [logOutputDirectory, setLogOutputDirectory] = useState(
    props.settings.logOutputDirectory
  );
  const hideOpenDirectoryDialog = props.platform === 'darwin';

  return (
    <div
      className={css`
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
        className={css`
          display: flex;
          align-items: end;
        `}
      >
        <TextField
          className={css`
            flex-grow: 1;
          `}
          styles={{
            fieldGroup: hideOpenDirectoryDialog
              ? {}
              : {
                borderRight: 'none',
                borderTopRightRadius: 0,
                borderBottomRightRadius: 0,
              },
          }}
          label="ログの出力先"
          disabled={!props.settings.logEnabled}
          value={logOutputDirectory}
          onChange={(_ev, newValue) => setLogOutputDirectory(newValue!!)}
          onBlur={() => {
            if (logOutputDirectory === props.settings.logOutputDirectory) {
              return;
            }
            props.onChange({ ...props.settings, logOutputDirectory });
          }}
        />
        <DefaultButton
          className={css`
            border-top-left-radius: 0;
            border-bottom-left-radius: 0;
            min-width: 0;
            ${hideOpenDirectoryDialog ? 'display: none;' : ''}
          `}
          iconProps={{ iconName: 'folderopen' }}
          disabled={!props.settings.logEnabled}
          onClick={async () => {
            const newLogOutputDirectory = (await dialog.open({
              defaultPath: props.settings.logOutputDirectory,
              directory: true,
            })) as string | null;
            if (
              newLogOutputDirectory == null ||
              newLogOutputDirectory === logOutputDirectory
            ) {
              return;
            }
            setLogOutputDirectory(newLogOutputDirectory);
            props.onChange({
              ...props.settings,
              logOutputDirectory: newLogOutputDirectory,
            });
          }}
        />
      </div>
      <div
        className={css`
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
      <div
        className={css`
          margin-top: 4ex;
        `}
      >
        アプリバージョン: v{props.version}
      </div>
    </div>
  );
}
