import { app } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { Event, listen } from '@tauri-apps/api/event';
import * as os from "@tauri-apps/plugin-os"
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
import OtherSettings from './components/OtherSettings';

const initialStatus: Status = {
  rtmp: 'idle',
};

function initialTab(ypConfigs: readonly YPConfig[], defaultSettings: Settings) {
  const doneGeneral = defaultSettings.generalSettings.channelName[0].length > 0;

  const ypSettings = defaultSettings.yellowPagesSettings;
  const usingHosts = [ypSettings.ipv4, ypSettings.ipv6]
    .map((x) => x.host)
    .filter((host) => host.length > 0);
  const doneYP =
    usingHosts.length > 0 &&
    ypConfigs
      .filter((ypConfig) => usingHosts.includes(ypConfig.host))
      .map((ypConfig) => ypConfig.termsURL)
      .every((termsURL) => ypSettings.agreedTerms[termsURL] != null);

  return doneGeneral && doneYP
    ? 'チャンネル情報'
    : doneGeneral
      ? 'YP 設定'
      : '基本設定';
}

export default function App(props: {
  ypConfigs: readonly YPConfig[];
  defaultSettings: Settings;
  contactStatus: { title: string; resCount: number };
}) {
  const [notifications, setNotifications] = useState<
    readonly {
      level: string;
      message: string;
    }[]
  >([]);
  const [settings, setSettings] = useState(props.defaultSettings);
  const [contactStatus, setContactStatus] = useState(props.contactStatus);
  const [_status, setStatus] = useState(initialStatus);
  const [platform, setPlatform] = useState('');
  const [version, setVersion] = useState('');

  useEffect(() => {
    const notifyPromise = listenWrapped(
      'notify',
      (ev: Event<{ level: string; message: string }>) => {
        setNotifications((notifications) => {
          // 連続で同じ内容を積まない
          const current = ev.payload;
          const last = notifications[notifications.length - 1];
          if (
            last != null &&
            current.level === last.level &&
            current.message === last.message
          ) {
            return notifications;
          }
          return [...notifications, ev.payload];
        });
      }
    );
    const pushSettingsPromise = listenWrapped(
      'push_settings',
      (ev: Event<Settings>) => {
        setSettings(ev.payload);
      }
    );
    const pushContactStatusPromise = listenWrapped(
      'push_contact_status',
      (ev: Event<{ title: string; resCount: number }>) => {
        setContactStatus(ev.payload);
      }
    );
    const statusPromise = listen('status', (ev: Event<Status>) => {
      setStatus(ev.payload);
    });

    // TODO: 配信中に終了しようとした時に確認ダイアログを出す
    // TODO: パラメーターの編集中に終了した時に内容を保存する

    (async () => {
      const platform = await os.platform();
      setPlatform(platform);
      const version = await app.getVersion();
      setVersion(version);
    })();

    return () => {
      notifyPromise.then((unlistenFn) => unlistenFn());
      pushContactStatusPromise.then((unlistenFn) => unlistenFn());
      pushSettingsPromise.then((unlistenFn) => unlistenFn());
      statusPromise.then((unlistenFn) => unlistenFn());
    };
  }, []);

  const [readedTerms, setReadedTerms] = useState<{ [key: string]: string }>(
    props.defaultSettings.yellowPagesSettings.agreedTerms
  );

  return (
    <>
      <TabContainer
        initialTab={initialTab(props.ypConfigs, props.defaultSettings)}
      >
        <TabContent label="基本設定">
          <GeneralSettings
            settings={settings.generalSettings}
            onChange={(generalSettings) => {
              invoke('put_settings', { generalSettings });
              setSettings((settings) => ({ ...settings, generalSettings }));
            }}
          />
        </TabContent>
        <TabContent label="YP 設定">
          <YellowPagesSettings
            ypConfigs={props.ypConfigs}
            settings={settings.yellowPagesSettings}
            readedTerms={readedTerms}
            onReadTerms={(termsURL, hash) => {
              setReadedTerms({ ...readedTerms, [termsURL]: hash });
            }}
            onChange={(yellowPagesSettings) => {
              invoke('put_settings', { yellowPagesSettings });
              setSettings((settings) => ({ ...settings, yellowPagesSettings }));
            }}
          />
        </TabContent>
        <TabContent label="チャンネル情報">
          <ChannelSettings
            settings={settings.channelSettings}
            contactStatus={contactStatus}
            onChange={(channelSettings) => {
              invoke('put_settings', { channelSettings });
              setSettings((settings) => ({ ...settings, channelSettings }));
            }}
          />
        </TabContent>
        <TabContent label="その他">
          <OtherSettings
            platform={platform}
            version={version}
            settings={settings.otherSettings}
            onChange={(otherSettings) => {
              invoke('put_settings', { otherSettings });
              setSettings((settings) => ({ ...settings, otherSettings }));
            }}
          />
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
