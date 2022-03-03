export enum PeerCastType {
  peerCastOriginal = 'PeerCastOriginal',
  peerCastStation = 'PeerCastStation',
}

export interface GeneralSettings {
  peerCastPort: number;
  channelName: readonly string[];
  rtmpListenPort: number;
}

export interface YellowPagesSettings {
  ipv4YpHost: string;
  ipv4YpGenrePrefix: string;
  ipv6YpHost: string;
  ipv6YpGenrePrefix: string;
}

export interface ChannelSettings {
  genre: readonly string[];
  desc: readonly string[];
  comment: readonly string[];
  contactUrl: readonly string[];
}

export default interface Settings {
  generalSettings: GeneralSettings;
  yellowPagesSettings: YellowPagesSettings;
  channelSettings: ChannelSettings;
}
