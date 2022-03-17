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
  settings: Settings;
}) {
  const [notifications, setNotifications] = useState<
    readonly {
      level: string;
      message: string;
    }[]
  >([]);

  useEffect(() => {
    const promise = listenWrapped(
      'notify',
      (ev: Event<{ level: string; message: string }>) => {
        setNotifications((notifications) => [...notifications, ev.payload]);
      }
    );
    return () => {
      promise.then((unlistenFn) => unlistenFn());
    };
  }, []);

  return (
    <>
      <TabContainer>
        <TabContent label="基本設定">
          <GeneralSettings defaultSettings={props.settings.generalSettings} />
        </TabContent>
        <TabContent label="YP 設定">
          <YellowPagesSettings
            ypConfigs={props.ypConfigs}
            defaultSettings={props.settings.yellowPagesSettings}
          />
        </TabContent>
        <TabContent label="チャンネル情報">
          <ChannelSettings defaultSettings={props.settings.channelSettings} />
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
