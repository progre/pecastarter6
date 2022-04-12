export enum PeerCastType {
  peerCastOriginal = 'PeerCastOriginal',
  peerCastStation = 'PeerCastStation',
}

export interface GeneralSettings {
  peerCastPort: number;
  peerCastRtmpPort: number;
  channelName: readonly string[];
  rtmpListenPort: number;
}

export interface EachYellowPagesSettings {
  host: string;
  hideListeners: boolean;
  namespace: string;
  portBandwidthCheck: 0 | 1 | 2 | 3;
  noLog: boolean;
  icon: string;
}

export interface YellowPagesSettings {
  ipv4: EachYellowPagesSettings;
  ipv6: EachYellowPagesSettings;
  agreedTerms: { [url: string]: string };
}

export interface ChannelContent {
  genre: string;
  desc: string;
}

export interface ChannelSettings {
  channelContentHistory: readonly ChannelContent[];
  genre: string;
  desc: string;
  comment: readonly string[];
  contactUrl: readonly string[];
}

export interface OtherSettings {
  logEnabled: boolean;
  logOutputDirectory: string;
}

export default interface Settings {
  generalSettings: GeneralSettings;
  yellowPagesSettings: YellowPagesSettings;
  channelSettings: ChannelSettings;
  otherSettings: OtherSettings;
}
