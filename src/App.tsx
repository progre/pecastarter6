import { invoke } from '@tauri-apps/api';
import { Event } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import Notification from './components/molecules/Notification';
import TabContainer, { TabContent } from './components/molecules/TabContainer';
import ChannelSettings from './components/ChannelSettings';
import GeneralSettings from './components/GeneralSettings';
import YellowPagesSettings from './components/YellowPagesSettings';
import Settings from './entities/Settings';
import YPConfig from './entities/YPConfig';
import listenWrapped from './utils/listenWrapped';

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
    return () => {
      notifyPromise.then((unlistenFn) => unlistenFn());
      pushSettingsPromise.then((unlistenFn) => unlistenFn());
    };
  }, []);

  return (
    <>
      <TabContainer>
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
