import { invoke } from '@tauri-apps/api';
import { Event, listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import { confirm } from '@tauri-apps/api/dialog';
import { useEffect, useState } from 'react';
import Notification from './components/molecules/Notification';
import TabContainer, { TabContent } from './components/molecules/TabContainer';
import ChannelSettings from './components/ChannelSettings';
import GeneralSettings from './components/GeneralSettings';
import YellowPagesSettings from './components/YellowPagesSettings';
import Settings from './entities/Settings';
import YPConfig from './entities/YPConfig';
import listenWrapped from './utils/listenWrapped';
import Status from './entities/Status';

const initialStatus: Status = {
  rtmp: 'idle',
};

export default function App(props: {
  ypConfigs: readonly YPConfig[];
  defaultSettings: Settings;
}) {
  const [notifications, setNotifications] = useState<
    readonly {
      level: string;
      message: string;
    }[]
  >([]);
  const [settings, setSettings] = useState(props.defaultSettings);
  const [status, setStatus] = useState(initialStatus);

  useEffect(() => {
    const notifyPromise = listenWrapped(
      'notify',
      (ev: Event<{ level: string; message: string }>) => {
        setNotifications((notifications) => [...notifications, ev.payload]);
      }
    );
    const pushSettingsPromise = listenWrapped(
      'push_settings',
      (ev: Event<Settings>) => {
        setSettings(ev.payload);
      }
    );
    const statusPromise = listen('status', (ev: Event<Status>) => {
      setStatus(ev.payload);
    });
    const closeRequestedPromise = appWindow.listen(
      'tauri://close-requested',
      async () => {
        if (
          status.rtmp !== 'streaming' ||
          (await confirm('アプリを終了するとエンコードが停止します。'))
        ) {
          // HACK: 閉じる前に onBlur を処理
          document
            .querySelector<HTMLButtonElement>('button[role="tab"]')!!
            .focus();
          appWindow.close();
        }
      }
    );
    return () => {
      notifyPromise.then((unlistenFn) => unlistenFn());
      pushSettingsPromise.then((unlistenFn) => unlistenFn());
      statusPromise.then((unlistenFn) => unlistenFn());
      closeRequestedPromise.then((unlistenFn) => unlistenFn());
    };
  }, []);

  const doneGeneral =
    props.defaultSettings.generalSettings.channelName[0].length > 0;

  const ypSettings = props.defaultSettings.yellowPagesSettings;
  const usingHosts = [ypSettings.ipv4, ypSettings.ipv6]
    .map((x) => x.host)
    .filter((host) => host.length > 0);
  const doneYP =
    usingHosts.length > 0 &&
    props.ypConfigs
      .filter((ypConfig) => usingHosts.includes(ypConfig.host))
      .map((ypConfig) => ypConfig.termsURL)
      .every((termsURL) => ypSettings.agreedTerms[termsURL] != null);

  const initialTab =
    doneGeneral && doneYP
      ? 'チャンネル情報'
      : doneGeneral
      ? 'YP 設定'
      : '基本設定';
  return (
    <>
      <TabContainer initialTab={initialTab}>
        <TabContent label="基本設定">
          <GeneralSettings defaultSettings={settings.generalSettings} />
        </TabContent>
        <TabContent label="YP 設定">
          <YellowPagesSettings
            ypConfigs={props.ypConfigs}
            settings={settings.yellowPagesSettings}
            onChange={(yellowPagesSettings) =>
              setSettings((settings) => ({ ...settings, yellowPagesSettings }))
            }
            onBlur={() => {
              invoke('set_yellow_pages_settings', {
                yellowPagesSettings: settings.yellowPagesSettings,
              });
            }}
          />
        </TabContent>
        <TabContent label="チャンネル情報">
          <ChannelSettings defaultSettings={settings.channelSettings} />
        </TabContent>
      </TabContainer>
      {notifications.length === 0 ? null : (
        <Notification
          level={notifications[0].level}
          message={notifications[0].message}
          onClickClose={() => {
            setNotifications((notifications) => notifications.slice(1));
          }}
        />
      )}
    </>
  );
}
